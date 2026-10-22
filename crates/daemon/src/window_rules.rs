//! Window rule evaluation and application: tile/float/ignore decisions and rule-driven enumeration.

use crate::config;
use crate::state::*;
use anyhow::Result;
use leopardwm_core_layout::{Rect, Workspace};
#[cfg(not(test))]
use leopardwm_platform_win32::enumerate_windows;
use leopardwm_platform_win32::{
    find_monitor_for_rect, get_process_executable, scale_px, MonitorId, WindowInfo,
};
use tracing::{debug, info, warn};

/// Width of a column moved or dragged onto another monitor, per
/// `behavior.new_width_on_monitor_move` (see `moved_column_width`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum MovedColumnWidth {
    /// The same share of the target viewport it had of the source viewport.
    Fraction(f64),
    /// The window rule's width in px, or `None` for the target's default width.
    Px(Option<i32>),
}

impl MovedColumnWidth {
    /// Size `workspace`'s focused column (the one just moved in).
    pub(crate) fn apply_to_focused_column(self, workspace: &mut Workspace, viewport_width: i32) {
        match self {
            Self::Fraction(fraction) => {
                workspace.restore_focused_column_width_fraction(fraction, viewport_width)
            }
            Self::Px(px) => {
                let width = px.unwrap_or(workspace.default_column_width());
                workspace.set_focused_column_width(width);
            }
        }
    }
}

impl AppState {
    /// Apply changed rule outcomes to managed windows, preserving manual overrides
    /// when the previous and current rules decide the same action.
    pub(crate) fn reapply_window_rules(&mut self, previous_rules: &[config::CompiledWindowRule]) {
        // Collect all managed windows with their current state
        let mut transitions: Vec<(u64, MonitorId, usize, config::WindowAction, bool)> = Vec::new();

        for (&monitor_id, ws_vec) in &self.workspaces {
            for (ws_idx, workspace) in ws_vec.iter().enumerate() {
                for wid in workspace.all_window_ids() {
                    let is_floating = workspace.is_floating(wid);
                    if let Some(win_info) = self.lookup_window_info(wid) {
                        let executable =
                            get_process_executable(win_info.process_id).unwrap_or_default();
                        let action = self.evaluate_window_rules(
                            &win_info.class_name,
                            &win_info.title,
                            &executable,
                            None,
                        );
                        let previous_action = previous_rules
                            .iter()
                            .find(|rule| {
                                rule.matches(
                                    &win_info.class_name,
                                    &win_info.title,
                                    &executable,
                                    None,
                                )
                            })
                            .map(|rule| rule.action)
                            .unwrap_or(config::WindowAction::Tile);
                        if action != previous_action {
                            transitions.push((wid, monitor_id, ws_idx, action, is_floating));
                        }
                    }
                }
            }
        }

        // Pre-compute floating rects before mutating workspaces (avoids borrow conflicts)
        let float_rects: std::collections::HashMap<u64, Rect> = transitions
            .iter()
            .filter(|(_, _, _, action, is_floating)| {
                *action == config::WindowAction::Float && !is_floating
            })
            .filter_map(|(wid, monitor_id, _, _, _)| {
                let win_info = self.lookup_window_info(*wid)?;
                let executable = get_process_executable(win_info.process_id).unwrap_or_default();
                let rect = self.get_floating_rect_from_rules(
                    &win_info.class_name,
                    &win_info.title,
                    &executable,
                    &win_info.rect,
                    Some(*monitor_id),
                    None,
                );
                Some((*wid, rect))
            })
            .collect();

        for (wid, monitor_id, ws_idx, action, is_floating) in transitions {
            match action {
                config::WindowAction::Float if !is_floating => {
                    // Currently tiled, should be floating — restore snap before moving
                    self.restore_snap_for_window(wid);
                    let viewport = self
                        .monitors
                        .get(&monitor_id)
                        .map(|m| m.work_area)
                        .unwrap_or_else(|| {
                            Rect::new(0, 0, FALLBACK_VIEWPORT_WIDTH, FALLBACK_VIEWPORT_HEIGHT)
                        });
                    let rect = float_rects.get(&wid).copied().unwrap_or_else(|| {
                        Rect::new(
                            viewport.x + (viewport.width - 800) / 2,
                            viewport.y + (viewport.height - 600) / 2,
                            800,
                            600,
                        )
                    });
                    if let Some(workspace) = self
                        .workspaces
                        .get_mut(&monitor_id)
                        .and_then(|v| v.get_mut(ws_idx))
                    {
                        let _ = workspace.remove_window(wid);
                        let _ = workspace.add_floating(wid, rect);
                        info!("Rule change: moved window {} to floating", wid);
                    }
                }
                config::WindowAction::Tile if is_floating => {
                    // Currently floating, should be tiled
                    if let Some(workspace) = self
                        .workspaces
                        .get_mut(&monitor_id)
                        .and_then(|v| v.get_mut(ws_idx))
                    {
                        workspace.unfloat_window(wid);
                        self.disable_snap_for_window(wid);
                        info!("Rule change: moved window {} to tiled", wid);
                    }
                }
                config::WindowAction::Ignore => {
                    // Should no longer be managed — restore snap and remove from workspace
                    self.restore_snap_for_window(wid);
                    if let Some(workspace) = self
                        .workspaces
                        .get_mut(&monitor_id)
                        .and_then(|v| v.get_mut(ws_idx))
                    {
                        if is_floating {
                            workspace.remove_floating(wid);
                        } else {
                            let _ = workspace.remove_window(wid);
                        }
                        self.window_managed_at.remove(&wid);
                        self.window_last_maximized_at.remove(&wid);
                        self.take_managed_lifetime_token(wid);
                        info!("Rule change: unmanaged window {} (ignore)", wid);
                    }
                }
                _ => {} // No change needed
            }
        }
    }

    pub(crate) fn preferred_new_window_monitor(
        &self,
        policy: config::NewWindowMonitor,
        sticky: bool,
    ) -> Option<MonitorId> {
        if policy == config::NewWindowMonitor::Focused
            && !sticky
            && self.monitors.contains_key(&self.focused_monitor)
        {
            Some(self.focused_monitor)
        } else {
            None
        }
    }

    pub(crate) fn floating_rect_for_admission(
        &self,
        window: &WindowInfo,
        executable: &str,
        monitor_id: MonitorId,
        use_preferred_monitor: bool,
    ) -> Rect {
        let mut rect = self.get_floating_rect_from_rules(
            &window.class_name,
            &window.title,
            executable,
            &window.rect,
            Some(monitor_id),
            Some(self.window_rule_size(&window.rect)),
        );
        if use_preferred_monitor {
            let monitor = &self.monitors[&monitor_id];
            if !monitor.contains_rect_center(&rect) {
                let area = monitor.work_area;
                rect.width = rect.width.min(area.width.max(1));
                rect.height = rect.height.min(area.height.max(1));
                rect.x = area.x + (area.width - rect.width) / 2;
                rect.y = area.y + (area.height - rect.height) / 2;
            }
        }
        rect
    }

    /// Enumerate windows and add them to the appropriate workspace based on position.
    pub(crate) fn enumerate_and_add_windows(&mut self) -> Result<usize> {
        let windows = self.windows_for_enumeration()?;
        let monitors: Vec<_> = self.monitors.values().cloned().collect();
        let mut added = 0;
        // Reconcile the previous replaced HWND before this iteration departs,
        // including when that window was skipped by the ignore gate or a rule.
        let mut pending_replaced: Option<u64> = None;

        for win_info in windows {
            if let Some(hwnd) = pending_replaced.take() {
                self.reconcile_replaced_lifetime_admission(hwnd);
            }
            // Recycle departs before the ignore gate and rules, matching
            // admission. The replacement is then evaluated like any new window.
            if self.depart_replaced_managed_lifetime(win_info.hwnd) {
                pending_replaced = Some(win_info.hwnd);
            }

            if matches!(
                self.temporary_ignore_gate(win_info.hwnd),
                crate::temporary_ignore::IgnoreGate::Block
            ) {
                continue;
            }
            let executable = get_process_executable(win_info.process_id).unwrap_or_default();

            self.size_ignored_windows.remove(&win_info.hwnd);
            // Restored and previously managed windows are not new admissions.
            if self.find_window_workspace(win_info.hwnd).is_some() {
                self.record_managed_lifetime_if_unrecorded(win_info.hwnd);
                self.record_managed_window_identity(&win_info);
                continue;
            }
            let matched = self.admission_rule(&win_info, &executable);
            let action = matched
                .map(|rule| rule.action)
                .unwrap_or(config::WindowAction::Tile);
            let rule_matched = matched.is_some();

            if action == config::WindowAction::Ignore {
                debug!(
                    "Ignoring window by rule: {} ({})",
                    win_info.title, win_info.class_name
                );
                continue;
            }

            let monitor_id = find_monitor_for_rect(&monitors, &win_info.rect)
                .map(|m| m.id)
                .unwrap_or(self.focused_monitor);

            // Get floating rect before borrowing workspace mutably (to avoid borrow conflict)
            let floating_rect = if action == config::WindowAction::Float {
                Some(self.get_floating_rect_from_rules(
                    &win_info.class_name,
                    &win_info.title,
                    &executable,
                    &win_info.rect,
                    Some(monitor_id),
                    Some(self.window_rule_size(&win_info.rect)),
                ))
            } else {
                None
            };

            // Elevated window the non-elevated daemon can't reposition (UIPI):
            // skip + notify instead of reserving a column we can't fill. Mirrors
            // the live-create path; covers windows already open at startup and
            // any seen via `lwm refresh`.
            if self.elevation_blocks_admission(
                win_info.hwnd,
                win_info.process_id,
                &win_info.title,
                &win_info.class_name,
            ) {
                continue;
            }

            if let Some(kind) = self.unmanaged_helper_kind(&win_info, rule_matched) {
                debug!(
                    "Leaving {} window unmanaged: {} ({})",
                    kind, win_info.title, win_info.class_name
                );
                continue;
            }

            // New windows land on the monitor's active workspace.
            let target_idx = self.active_workspace_idx(monitor_id);
            let _ = self.ensure_workspace_exists(monitor_id, target_idx);
            if let Some(workspace) = self
                .workspaces
                .get_mut(&monitor_id)
                .and_then(|v| v.get_mut(target_idx))
            {
                match action {
                    config::WindowAction::Float => {
                        // Use rule dimensions or default to centered 800x600 window
                        let rule_rect = floating_rect.unwrap_or_else(|| {
                            let viewport = self
                                .monitors
                                .get(&monitor_id)
                                .map(|m| m.work_area)
                                .unwrap_or_else(|| {
                                    Rect::new(
                                        0,
                                        0,
                                        FALLBACK_VIEWPORT_WIDTH,
                                        FALLBACK_VIEWPORT_HEIGHT,
                                    )
                                });
                            Rect::new(
                                viewport.x + (viewport.width - 800) / 2,
                                viewport.y + (viewport.height - 600) / 2,
                                800,
                                600,
                            )
                        });

                        match workspace.add_floating(win_info.hwnd, rule_rect) {
                            Ok(()) => {
                                self.window_managed_at
                                    .insert(win_info.hwnd, std::time::Instant::now());
                                self.record_managed_lifetime(win_info.hwnd, None);
                                self.record_managed_window_identity(&win_info);
                                info!(
                                    "Added floating window: {} ({}) to monitor {} - {}x{}",
                                    win_info.title,
                                    win_info.class_name,
                                    monitor_id,
                                    rule_rect.width,
                                    rule_rect.height
                                );
                                added += 1;
                            }
                            Err(e) => {
                                warn!("Failed to add floating window {}: {}", win_info.hwnd, e);
                            }
                        }
                    }
                    config::WindowAction::Tile => {
                        match workspace.insert_window(win_info.hwnd, None) {
                            Ok(()) => {
                                self.window_managed_at
                                    .insert(win_info.hwnd, std::time::Instant::now());
                                self.record_managed_lifetime(win_info.hwnd, None);
                                self.record_managed_window_identity(&win_info);
                                self.disable_snap_for_window(win_info.hwnd);
                                info!(
                                    "Added tiled window: {} ({}) to monitor {} - {}x{}",
                                    win_info.title,
                                    win_info.class_name,
                                    monitor_id,
                                    win_info.rect.width,
                                    win_info.rect.height
                                );
                                added += 1;
                            }
                            Err(e) => {
                                warn!("Failed to add window {}: {}", win_info.hwnd, e);
                            }
                        }
                    }
                    config::WindowAction::Ignore => unreachable!(), // Handled above
                }
            }
        }

        if let Some(hwnd) = pending_replaced {
            self.reconcile_replaced_lifetime_admission(hwnd);
        }

        Ok(added)
    }

    /// Leave dialog-like windows, or untitled frameless fixed-size windows not
    /// covering a monitor when topology is known, unmanaged. A matching user
    /// rule overrides this.
    pub(crate) fn unmanaged_helper_kind(
        &self,
        window: &WindowInfo,
        rule_matched: bool,
    ) -> Option<&'static str> {
        if rule_matched {
            return None;
        }
        if leopardwm_platform_win32::is_dialog_like_window(window.hwnd) {
            return Some("dialog-like");
        }
        if window.title.trim().is_empty()
            && !self.monitors.is_empty()
            && leopardwm_platform_win32::is_frameless_fixed_size_window(window.hwnd)
            && !self
                .monitors
                .values()
                .any(|monitor| crate::event_handler::rect_covers_monitor(window.rect, monitor))
        {
            return Some("frameless fixed-size");
        }
        None
    }

    fn windows_for_enumeration(&self) -> Result<Vec<WindowInfo>> {
        #[cfg(test)]
        {
            Ok(self.injected_enumerated_windows.clone().unwrap_or_default())
        }
        #[cfg(not(test))]
        Ok(enumerate_windows()?)
    }

    /// Match once at admission and remember size-ignored windows for size/title retries.
    pub(crate) fn admission_rule(
        &mut self,
        window: &WindowInfo,
        executable: &str,
    ) -> Option<&config::CompiledWindowRule> {
        self.size_ignored_windows.remove(&window.hwnd);
        let size = Some(self.window_rule_size(&window.rect));
        let matched = self
            .compiled_rules
            .iter()
            .find(|rule| rule.matches(&window.class_name, &window.title, executable, size));
        if matched.is_some_and(|rule| {
            rule.action == config::WindowAction::Ignore && rule.has_size_condition()
        }) {
            self.size_ignored_windows.insert(window.hwnd);
        }
        matched
    }

    pub(crate) fn window_rule_size(&self, rect: &Rect) -> (i32, i32) {
        let scale = self
            .monitors
            .values()
            .find(|monitor| monitor.contains_rect_center(rect))
            .map(|monitor| monitor.scale_factor)
            .unwrap_or(1.0);
        (
            scale_px(rect.width, 1.0 / scale),
            scale_px(rect.height, 1.0 / scale),
        )
    }

    /// Evaluate window rules and return the action for a window.
    pub(crate) fn evaluate_window_rules(
        &self,
        class_name: &str,
        title: &str,
        executable: &str,
        size: Option<(i32, i32)>,
    ) -> config::WindowAction {
        self.matched_rule(class_name, title, executable, size)
            .map(|r| r.action)
            .unwrap_or(config::WindowAction::Tile)
    }

    /// A rule's `column_width` in px on `monitor_id`, or `None` when the rule
    /// sets none for that monitor (the monitor's default width applies).
    pub(crate) fn rule_column_width_px(
        &self,
        rule_column_width: Option<&config::CompiledColumnWidth>,
        monitor_id: MonitorId,
        viewport_width: i32,
    ) -> Option<i32> {
        let display_idx = self.display_index_for(monitor_id);
        rule_column_width
            .and_then(|cw| cw.resolve(display_idx))
            .map(|f| ((f * f64::from(viewport_width)).round() as i32).max(100))
    }

    /// Column width in px for a managed window landing on `monitor_id` (moved
    /// or dragged there): the same width a newly opened window would get, so
    /// its matched rule's `column_width`, or `None` for the monitor's default.
    pub(crate) fn landing_column_width_px(&self, hwnd: u64, monitor_id: MonitorId) -> Option<i32> {
        let info = self.lookup_window_info(hwnd)?;
        let executable = get_process_executable(info.process_id).unwrap_or_default();
        let rule = self.matched_rule(
            &info.class_name,
            &info.title,
            &executable,
            Some(self.window_rule_size(&info.rect)),
        )?;
        self.rule_column_width_px(
            rule.column_width.as_ref(),
            monitor_id,
            self.viewport_width_for(monitor_id),
        )
    }

    /// Width of the column `hwnd` gets when moved or dragged onto another
    /// monitor as a new column, per `behavior.new_width_on_monitor_move`, or
    /// `None` in `Upstream` mode (and when `target_monitor` is the source
    /// monitor), where the caller keeps main's behavior. Call it before
    /// `hwnd` leaves the source workspace; if it already left (a drag that
    /// took it out of a stacked column), `source_peers` locate that column.
    pub(crate) fn moved_column_width(
        &self,
        hwnd: u64,
        source_monitor: MonitorId,
        source_ws_idx: usize,
        source_peers: &[u64],
        target_monitor: MonitorId,
    ) -> Option<MovedColumnWidth> {
        if target_monitor == source_monitor {
            return None;
        }
        match self.config.behavior.new_width_on_monitor_move {
            config::NewWidthOnMonitorMove::Upstream => None,
            config::NewWidthOnMonitorMove::SameProportionAsSourceMonitor => {
                let ws = self.workspaces.get(&source_monitor)?.get(source_ws_idx)?;
                let viewport_width = self.viewport_width_for(source_monitor);
                std::iter::once(&hwnd)
                    .chain(source_peers)
                    .find_map(|&w| ws.column_width_fraction_for_window(w, viewport_width))
                    .map(MovedColumnWidth::Fraction)
            }
            config::NewWidthOnMonitorMove::WindowRuleWidthOrDefaultWidth => Some(
                MovedColumnWidth::Px(self.landing_column_width_px(hwnd, target_monitor)),
            ),
        }
    }

    /// The first window rule matching this window, if any (first match wins).
    pub(crate) fn matched_rule(
        &self,
        class_name: &str,
        title: &str,
        executable: &str,
        size: Option<(i32, i32)>,
    ) -> Option<&config::CompiledWindowRule> {
        self.compiled_rules
            .iter()
            .find(|rule| rule.matches(class_name, title, executable, size))
    }

    /// Get the floating rect for a window based on rules.
    ///
    /// Rule-defined `width`/`height` are config values (logical pixels) and
    /// are scaled by the monitor's DPI factor. The `monitor_id` parameter
    /// is used to look up the scale factor; pass `None` to skip scaling.
    pub(crate) fn get_floating_rect_from_rules(
        &self,
        class_name: &str,
        title: &str,
        executable: &str,
        original_rect: &leopardwm_core_layout::Rect,
        monitor_id: Option<MonitorId>,
        size: Option<(i32, i32)>,
    ) -> leopardwm_core_layout::Rect {
        let scale = monitor_id
            .and_then(|id| self.monitors.get(&id))
            .map(|m| m.scale_factor)
            .unwrap_or(1.0);
        for rule in &self.compiled_rules {
            if rule.matches(class_name, title, executable, size) {
                // Only scale rule-provided dimensions (config logical pixels).
                // If a dimension is not specified, use the original rect value
                // which is already in physical pixels from the OS.
                let width = rule
                    .width
                    .map(|w| scale_px(w, scale))
                    .unwrap_or(original_rect.width);
                let height = rule
                    .height
                    .map(|h| scale_px(h, scale))
                    .unwrap_or(original_rect.height);
                return leopardwm_core_layout::Rect::new(
                    original_rect.x,
                    original_rect.y,
                    width,
                    height,
                );
            }
        }
        *original_rect
    }
}
