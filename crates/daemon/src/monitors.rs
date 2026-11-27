use crate::helpers::ScaledLayoutParams;
use crate::state::*;
use leopardwm_core_layout::{Rect, Workspace};
use leopardwm_ipc::log_text;
use leopardwm_platform_win32::{MonitorId, MonitorInfo};
use std::collections::{HashMap, HashSet};
use tracing::{info, warn};

pub(crate) fn apply_reconciled_geometry(
    workspaces: &mut [Workspace],
    params: &ScaledLayoutParams,
    source_viewport_width: i32,
    destination_viewport_width: i32,
) {
    for (ws_idx, workspace) in workspaces.iter_mut().enumerate() {
        let old_gap = workspace.gap();
        let (old_ol, old_or, _, _) = workspace.outer_gaps();
        params.apply_to(workspace, ws_idx);
        if workspace.rescale_column_widths(
            old_gap,
            old_ol,
            old_or,
            source_viewport_width,
            destination_viewport_width,
        ) {
            workspace.ensure_focused_visible(destination_viewport_width);
        }
    }
}

impl AppState {
    pub(crate) fn restore_stashed_monitor_layout(
        &mut self,
        monitor: &MonitorInfo,
        mut stash: StashedMonitorLayout,
    ) -> i32 {
        let allowed: HashSet<u64> = self
            .workspaces
            .values()
            .flatten()
            .flat_map(|ws| ws.all_window_ids())
            .collect();
        let mut dropped = 0;
        for ws in &mut stash.workspaces {
            for wid in ws.all_window_ids() {
                if !allowed.contains(&wid) {
                    if ws.is_floating(wid) {
                        ws.remove_floating(wid);
                    } else {
                        let _ = ws.remove_window(wid);
                    }
                    dropped += 1;
                }
            }
            self.config.apply_monitor_rules(&monitor.device_name, ws);
            ws.set_center_columns(self.config.center_columns_for(&monitor.device_name).into());
            ws.ensure_focused_visible(stash.source_viewport_width);
        }
        let returning: HashSet<u64> = stash
            .workspaces
            .iter()
            .flat_map(|w| w.all_window_ids())
            .collect();
        for ws_vec in self.workspaces.values_mut() {
            for ws in ws_vec {
                for &wid in &returning {
                    let _ = ws.remove_window(wid);
                    ws.remove_floating(wid);
                }
            }
        }
        let clamped = stash
            .active_workspace
            .min(stash.workspaces.len().saturating_sub(1));
        info!("Restored {} stashed workspace(s) for reconnected monitor {} ({}); dropped {} window(s)",
            stash.workspaces.len(), monitor.id, log_text::name(&monitor.device_name), dropped);
        self.workspaces.insert(monitor.id, stash.workspaces);
        self.active_workspace.insert(monitor.id, clamped);
        stash.source_viewport_width
    }

    pub(crate) fn forget_stashed_windows(&mut self, windows: &[u64]) {
        for stash in self.stashed_monitor_layouts.values_mut() {
            for ws in &mut stash.workspaces {
                for &wid in windows {
                    let _ = ws.remove_window(wid);
                    ws.remove_floating(wid);
                }
            }
        }
        self.stashed_monitor_layouts.retain(|_, stash| {
            stash
                .workspaces
                .iter()
                .any(|ws| !ws.all_window_ids().is_empty())
        });
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn reconcile_monitors(&mut self, new_monitors: Vec<MonitorInfo>) {
        let new_ids: HashSet<MonitorId> = new_monitors.iter().map(|m| m.id).collect();
        let old_ids: HashSet<MonitorId> = self.monitors.keys().copied().collect();
        let old_viewport_widths: HashMap<MonitorId, i32> = self
            .monitors
            .iter()
            .map(|(&id, monitor)| (id, monitor_viewport_width(monitor)))
            .collect();
        let mut source_viewport_widths: HashMap<MonitorId, i32> = new_monitors
            .iter()
            .filter_map(|monitor| {
                old_viewport_widths
                    .get(&monitor.id)
                    .map(|&width| (monitor.id, width))
            })
            .collect();

        // Detect HMONITOR handle changes without physical topology change
        // (e.g., contrast theme switch). Match old→new by device_name and
        // re-key workspace data instead of destroying and recreating it.
        if new_ids != old_ids && new_monitors.len() == self.monitors.len() {
            let old_by_name: HashMap<&str, MonitorId> = self
                .monitors
                .values()
                .map(|m| (m.device_name.as_str(), m.id))
                .collect();
            let new_by_name: HashMap<&str, MonitorId> = new_monitors
                .iter()
                .map(|m| (m.device_name.as_str(), m.id))
                .collect();

            let remap: HashMap<MonitorId, MonitorId> = old_by_name
                .iter()
                .filter_map(|(name, &old_id)| new_by_name.get(name).map(|&new_id| (old_id, new_id)))
                .collect();

            if remap.len() == self.monitors.len() {
                info!(
                    "Monitor handles changed without topology change — re-keying {} workspace(s)",
                    remap.len()
                );
                for (&old_id, &new_id) in &remap {
                    if let Some(&source_width) = old_viewport_widths.get(&old_id) {
                        source_viewport_widths.insert(new_id, source_width);
                    }
                    if old_id != new_id {
                        if let Some(ws) = self.workspaces.remove(&old_id) {
                            self.workspaces.insert(new_id, ws);
                        }
                        if let Some(idx) = self.active_workspace.remove(&old_id) {
                            self.active_workspace.insert(new_id, idx);
                        }
                        if self.focused_monitor == old_id {
                            self.focused_monitor = new_id;
                        }
                    }
                }
                self.monitors = new_monitors.into_iter().map(|m| (m.id, m)).collect();
                // Re-apply scaled gaps in case DPI or work area changed
                for (&monitor_id, ws_vec) in self.workspaces.iter_mut() {
                    let scale = self
                        .monitors
                        .get(&monitor_id)
                        .map(|m| m.scale_factor)
                        .unwrap_or(1.0);
                    let viewport_width = self
                        .monitors
                        .get(&monitor_id)
                        .map(monitor_viewport_width)
                        .unwrap_or(FALLBACK_VIEWPORT_WIDTH);
                    let device_name = monitor_device_name(&self.monitors, monitor_id);
                    let params = ScaledLayoutParams::from_config(
                        &self.config,
                        scale,
                        viewport_width,
                        device_name,
                    );
                    let source_viewport_width = source_viewport_widths
                        .get(&monitor_id)
                        .copied()
                        .unwrap_or(viewport_width);
                    apply_reconciled_geometry(
                        ws_vec,
                        &params,
                        source_viewport_width,
                        viewport_width,
                    );
                    for workspace in ws_vec.iter_mut() {
                        self.config.apply_monitor_rules(device_name, workspace);
                    }
                }
                return;
            }
        }

        let primary_id = new_monitors
            .iter()
            .find(|m| m.is_primary)
            .or_else(|| new_monitors.first())
            .map(|m| m.id);

        // Handle added monitors - create new workspaces FIRST so migration
        // targets exist even when all old monitors are replaced with new ones.
        for monitor in &new_monitors {
            if !old_ids.contains(&monitor.id) {
                if let Some(stash) = self.stashed_monitor_layouts.remove(&monitor.device_name) {
                    let source_width = self.restore_stashed_monitor_layout(monitor, stash);
                    source_viewport_widths.insert(monitor.id, source_width);
                    continue;
                }
                // Same-pass handle change: the same physical monitor (matched by
                // device_name) is being removed elsewhere in this reconcile with a
                // different HMONITOR. Adopt its live layout directly instead of
                // recreating it, generalizing the count-unchanged re-key branch to
                // the count-changed case (e.g. a dock event that also adds a
                // monitor). The removed loop then finds nothing to migrate for it.
                let adopt_from = old_ids.iter().copied().find(|&oid| {
                    !new_ids.contains(&oid)
                        && self
                            .monitors
                            .get(&oid)
                            .is_some_and(|m| m.device_name == monitor.device_name)
                });
                if let Some(old_id) = adopt_from {
                    if let Some(ws) = self.workspaces.remove(&old_id) {
                        if let Some(&source_width) = old_viewport_widths.get(&old_id) {
                            source_viewport_widths.insert(monitor.id, source_width);
                        }
                        let idx = self.active_workspace.remove(&old_id).unwrap_or(0);
                        let clamped = idx.min(ws.len().saturating_sub(1));
                        if self.focused_monitor == old_id {
                            self.focused_monitor = monitor.id;
                        }
                        info!(
                            "Adopted live layout from monitor {} to {} ({}) across a handle change",
                            old_id,
                            monitor.id,
                            log_text::name(&monitor.device_name)
                        );
                        self.workspaces.insert(monitor.id, ws);
                        self.active_workspace.insert(monitor.id, clamped);
                        continue;
                    }
                }
                let mvw = monitor_viewport_width(monitor);
                source_viewport_widths.insert(monitor.id, mvw);
                let params = ScaledLayoutParams::from_config(
                    &self.config,
                    monitor.scale_factor,
                    mvw,
                    &monitor.device_name,
                );
                let mut workspace = Workspace::with_directional_gaps(
                    params.gap,
                    params.outer_gap_left,
                    params.outer_gap_right,
                    params.outer_gap_top,
                    params.outer_gap_bottom,
                );
                params.apply_to(&mut workspace, 0);
                self.config
                    .apply_monitor_rules(&monitor.device_name, &mut workspace);
                workspace.set_center_columns(
                    self.config.center_columns_for(&monitor.device_name).into(),
                );
                workspace.set_reduce_motion(self.reduce_motion);
                workspace.set_scroll_animation(
                    self.config.animation.scroll_duration_ms,
                    self.config.animation.easing,
                );
                self.workspaces.insert(monitor.id, vec![workspace]);
                self.active_workspace.insert(monitor.id, 0);
                info!("Created workspace for new monitor {}", monitor.id);
            }
        }

        for removed_id in old_ids.difference(&new_ids) {
            // The stable device_name and active index, captured before the
            // monitor info is dropped, so a reconnect can restore this layout.
            let device_name = self.monitors.get(removed_id).map(|m| m.device_name.clone());
            let source_viewport_width = old_viewport_widths[removed_id];
            let active_idx = self.active_workspace.get(removed_id).copied().unwrap_or(0);
            if let Some(old_ws_vec) = self.workspaces.remove(removed_id) {
                let mut tiled_window_ids = Vec::new();
                let mut floating_windows = Vec::new();
                let mut minimized_ids = Vec::new();
                for old_workspace in &old_ws_vec {
                    for row in old_workspace.rows() {
                        for col in row.columns() {
                            for &wid in col.windows() {
                                tiled_window_ids.push(wid);
                                if old_workspace.is_minimized(wid) {
                                    minimized_ids.push(wid);
                                }
                            }
                        }
                    }
                    for fw in old_workspace.floating_windows() {
                        floating_windows.push((fw.id, fw.rect));
                        if old_workspace.is_minimized(fw.id) {
                            minimized_ids.push(fw.id);
                        }
                    }
                }
                if let Some(primary) = primary_id {
                    let primary_active_idx = self.active_workspace_idx(primary);
                    let source_wa = self.monitors.get(removed_id).map(|m| m.work_area);
                    let target_wa = self.monitors.get(&primary).map(|m| m.work_area);
                    if let Some(primary_ws) = self
                        .workspaces
                        .get_mut(&primary)
                        .and_then(|v| v.get_mut(primary_active_idx))
                    {
                        for window_id in &tiled_window_ids {
                            if let Err(e) = primary_ws.insert_window(*window_id, None) {
                                warn!("Failed to migrate tiled window {}: {}", window_id, e);
                            }
                        }
                        for (wid, rect) in &floating_windows {
                            let translated = match (source_wa, target_wa) {
                                (Some(src), Some(tgt)) => {
                                    let dx = tgt.x - src.x;
                                    let dy = tgt.y - src.y;
                                    let max_x = (tgt.x + tgt.width - rect.width).max(tgt.x);
                                    let max_y = (tgt.y + tgt.height - rect.height).max(tgt.y);
                                    leopardwm_core_layout::Rect::new(
                                        (rect.x + dx).clamp(tgt.x, max_x),
                                        (rect.y + dy).clamp(tgt.y, max_y),
                                        rect.width,
                                        rect.height,
                                    )
                                }
                                _ => *rect,
                            };
                            if let Err(e) = primary_ws.add_floating(*wid, translated) {
                                warn!("Failed to migrate floating window {}: {}", wid, e);
                            }
                        }
                        for wid in &minimized_ids {
                            primary_ws.mark_minimized(*wid);
                        }
                        info!(
                            "Migrated {} tiled + {} floating windows from removed monitor {} to primary",
                            tiled_window_ids.len(),
                            floating_windows.len(),
                            removed_id
                        );
                    }
                }
                // Stash this monitor's layout by stable device_name so a
                // reconnect restores it; the windows migrated above stay on
                // primary meanwhile and are pulled back when it returns.
                if let Some(name) = device_name {
                    let owned: HashSet<u64> = self
                        .stashed_monitor_layouts
                        .values()
                        .flat_map(|stash| &stash.workspaces)
                        .flat_map(|ws| ws.all_window_ids())
                        .collect();
                    let mut stashed_ws = old_ws_vec;
                    for ws in &mut stashed_ws {
                        for wid in ws.all_window_ids() {
                            if owned.contains(&wid) {
                                let _ = ws.remove_window(wid);
                                ws.remove_floating(wid);
                            }
                        }
                    }
                    if stashed_ws.iter().any(|ws| !ws.all_window_ids().is_empty()) {
                        self.stashed_monitor_layouts.insert(
                            name,
                            StashedMonitorLayout {
                                workspaces: stashed_ws,
                                active_workspace: active_idx,
                                source_viewport_width,
                            },
                        );
                    }
                }
            }
            self.active_workspace.remove(removed_id);
            self.monitors.remove(removed_id);
        }

        self.monitors = new_monitors.into_iter().map(|m| (m.id, m)).collect();

        // Re-apply scaled gaps to ALL existing workspaces — monitor DPI or work
        // area may have changed even if the monitor ID stayed the same (e.g.,
        // Windows scaling change, or work area resize from taskbar move).
        for (&monitor_id, ws_vec) in self.workspaces.iter_mut() {
            let scale = self
                .monitors
                .get(&monitor_id)
                .map(|m| m.scale_factor)
                .unwrap_or(1.0);
            let viewport_width = self
                .monitors
                .get(&monitor_id)
                .map(monitor_viewport_width)
                .unwrap_or(FALLBACK_VIEWPORT_WIDTH);
            let device_name = monitor_device_name(&self.monitors, monitor_id);
            let params = ScaledLayoutParams::from_config(
                &self.config,
                scale,
                viewport_width,
                device_name,
            );

            let source_viewport_width = source_viewport_widths
                .get(&monitor_id)
                .copied()
                .unwrap_or(viewport_width);
            apply_reconciled_geometry(ws_vec, &params, source_viewport_width, viewport_width);
            for workspace in ws_vec.iter_mut() {
                self.config.apply_monitor_rules(device_name, workspace);
            }
        }

        if !self.monitors.contains_key(&self.focused_monitor) {
            self.focused_monitor = primary_id.unwrap_or(0);
        }
    }

    /// Resync minimized flags and park inactive-workspace windows for startup and resume.
    /// Pause still resyncs but skips parking and fullscreen observation.
    pub(crate) fn prepare_inactive_workspace_windows(&mut self) {
        self.resync_minimized_from_os();
        self.park_inactive_workspace_windows();
    }

    pub(crate) fn park_inactive_workspace_windows(&mut self) {
        // Shutdown recovers parked windows; apply_layout only places active workspaces.
        let mut inactive_windows = Vec::new();
        for (&monitor, workspaces) in &self.workspaces {
            let active = self.active_workspace_idx(monitor);
            for (index, workspace) in workspaces.iter().enumerate() {
                if index != active {
                    inactive_windows.extend(workspace.all_window_ids());
                }
            }
        }
        for wid in inactive_windows {
            self.park_inactive_workspace_window(wid);
        }
    }

    pub(crate) fn park_inactive_workspace_window(&mut self, wid: u64) -> bool {
        if self.paused
            || self
                .find_window_workspace(wid)
                .is_some_and(|(monitor, index)| self.workspaces[&monitor][index].is_minimized(wid))
            || self.is_application_fullscreen(wid)
        {
            return false;
        }
        if let Some(session) = self.observe_current_application_fullscreen(wid) {
            self.application_fullscreen.insert(wid, session);
            return false;
        }
        #[cfg(test)]
        {
            self.inactive_workspace_park_requests.push(wid);
            if !self.injected_native_offscreen_enabled {
                return true;
            }
        }
        if let Err(e) = leopardwm_platform_win32::queue_window_offscreen(wid) {
            warn!("Failed to park inactive workspace window {:#x}: {}", wid, e);
            return false;
        }
        true
    }

    /// Reconcile every managed window's minimized flag with the OS.
    ///
    /// A monitor sleep/wake cycle desyncs this: a stashed workspace restored on
    /// wake carries the pre-sleep minimized state, but Windows un-minimizes the
    /// windows when the monitor returns and those restore events land on the
    /// migrated copies (on the primary), which are then discarded. The result is
    /// a window that is visible on screen but still flagged minimized, so it is
    /// never tiled. The stale flag also persists in saved layout state across a
    /// restart, so this runs both after a display-change reconcile and once at
    /// startup, before the first layout is applied.
    pub(crate) fn resync_minimized_from_os(&mut self) {
        self.resync_minimized_with(leopardwm_platform_win32::window_minimized_state);
    }

    pub(crate) fn resync_minimized_with(&mut self, os_minimized: impl Fn(u64) -> Option<bool>) {
        for ws_vec in self.workspaces.values_mut() {
            for ws in ws_vec.iter_mut() {
                for wid in ws.all_window_ids() {
                    match os_minimized(wid) {
                        Some(true) if !ws.is_minimized(wid) => {
                            ws.mark_minimized(wid);
                        }
                        Some(false) if ws.is_minimized(wid) => {
                            ws.mark_restored(wid);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub(crate) fn monitor_move_target(&self) -> Option<u64> {
        // Prefer the OS-foreground window so floating windows can be moved too.
        let ws = self.focused_workspace()?;
        self.previous_focused_hwnd
            .filter(|&hwnd| ws.contains_window(hwnd))
            .or_else(|| ws.focused_window())
    }

    /// Move the focused window to another monitor using an all-or-nothing update.
    pub(crate) fn move_focused_window_to_monitor_transactional(
        &mut self,
        target_monitor: MonitorId,
    ) -> std::result::Result<Option<u64>, String> {
        let Some(window_id) = self.monitor_move_target() else {
            return Ok(None);
        };
        self.move_window_to_monitor_transactional(window_id, target_monitor)
    }

    /// Move a window on the focused monitor's active workspace to another monitor.
    ///
    /// The source and target workspaces are mutated on cloned snapshots first and
    /// committed only if both remove and insert operations succeed.
    pub(crate) fn move_window_to_monitor_transactional(
        &mut self,
        window_id: u64,
        target_monitor: MonitorId,
    ) -> std::result::Result<Option<u64>, String> {
        let source_monitor = self.focused_monitor;
        if source_monitor == target_monitor {
            return Ok(None);
        }
        let src_idx = self.active_workspace_idx(source_monitor);
        let tgt_idx = self.active_workspace_idx(target_monitor);

        let Some(source_ws_vec) = self.workspaces.get(&source_monitor) else {
            return Err(format!(
                "Source workspace missing for monitor {}",
                source_monitor
            ));
        };
        let Some(mut source_workspace) = source_ws_vec.get(src_idx).cloned() else {
            return Err(format!(
                "Source workspace index {} missing for monitor {}",
                src_idx, source_monitor
            ));
        };
        let Some(target_ws_vec) = self.workspaces.get(&target_monitor) else {
            return Err(format!(
                "Target workspace missing for monitor {}",
                target_monitor
            ));
        };
        let Some(mut target_workspace) = target_ws_vec.get(tgt_idx).cloned() else {
            return Err(format!(
                "Target workspace index {} missing for monitor {}",
                tgt_idx, target_monitor
            ));
        };

        let is_floating = source_workspace.is_floating(window_id);
        if is_floating {
            let rect = source_workspace
                .floating_windows()
                .iter()
                .find(|f| f.id == window_id)
                .map(|f| f.rect)
                .unwrap_or(leopardwm_core_layout::Rect::new(0, 0, 800, 600));
            let translated_rect = match (
                self.monitors.get(&source_monitor),
                self.monitors.get(&target_monitor),
            ) {
                (Some(src_mon), Some(tgt_mon)) => {
                    let dx = tgt_mon.work_area.x - src_mon.work_area.x;
                    let dy = tgt_mon.work_area.y - src_mon.work_area.y;
                    let max_x = (tgt_mon.work_area.x + tgt_mon.work_area.width - rect.width)
                        .max(tgt_mon.work_area.x);
                    let max_y = (tgt_mon.work_area.y + tgt_mon.work_area.height - rect.height)
                        .max(tgt_mon.work_area.y);
                    leopardwm_core_layout::Rect::new(
                        (rect.x + dx).clamp(tgt_mon.work_area.x, max_x),
                        (rect.y + dy).clamp(tgt_mon.work_area.y, max_y),
                        rect.width,
                        rect.height,
                    )
                }
                _ => rect,
            };
            source_workspace.remove_floating(window_id);
            target_workspace
                .add_floating(window_id, translated_rect)
                .map_err(|e| format!("Failed to add floating window to target: {}", e))?;
        } else {
            // In "same proportion" mode the exact fraction stays cached on
            // the column, so moving back and forth doesn't drift.
            let new_width =
                self.moved_column_width(window_id, source_monitor, src_idx, &[], target_monitor);
            source_workspace
                .remove_window(window_id)
                .map_err(|e| format!("Failed to remove window: {}", e))?;
            target_workspace
                .insert_window(window_id, None)
                .map_err(|e| format!("Failed to add window to target: {}", e))?;
            if let Some(new_width) = new_width {
                new_width.apply_to_focused_column(
                    &mut target_workspace,
                    self.viewport_width_for(target_monitor),
                );
            }
        }

        let target_viewport = self.viewport_width_for(target_monitor);
        if !is_floating {
            // Cancel any in-flight scroll animation so ensure_focused_visible's
            // scroll_offset is what compute_placements_animated actually reads.
            // If an animation is active, effective_scroll_offset() returns the
            // animation's stale position instead of the newly computed offset.
            let source_viewport = self.viewport_width_for(source_monitor);
            source_workspace.stop_animation();
            source_workspace.ensure_focused_visible(source_viewport);
            target_workspace.stop_animation();
            target_workspace.ensure_focused_visible(target_viewport);
            source_workspace.reconcile_scroll_bounds(self.viewport_width_for(source_monitor));
        }

        self.workspaces.get_mut(&source_monitor).unwrap()[src_idx] = source_workspace;
        self.workspaces.get_mut(&target_monitor).unwrap()[tgt_idx] = target_workspace;
        self.forget_stashed_windows(&[window_id]);
        self.focused_monitor = target_monitor;
        Ok(Some(window_id))
    }

    /// The monitor currently under the mouse cursor, if the cursor position
    /// can be read and falls within a known monitor's bounds.
    ///
    /// Always `None` under `#[cfg(test)]`: the real cursor position on the
    /// machine running the test suite is not hermetic, and test monitor
    /// fixtures commonly use realistic screen rects (e.g. 1920x1080) that
    /// could coincidentally contain it, making assertions depend on where
    /// the physical mouse happens to be.
    pub(crate) fn monitor_under_cursor(&self) -> Option<MonitorId> {
        #[cfg(test)]
        {
            None
        }
        #[cfg(not(test))]
        {
            let (x, y) = leopardwm_platform_win32::get_cursor_pos()?;
            self.monitors
                .values()
                .find(|m| m.contains_point(x, y))
                .map(|m| m.id)
        }
    }

    /// Select the monitor for a newly created window under `policy` (the
    /// effective `new_window_monitor`). `Cursor` uses the cursor monitor
    /// first; otherwise (and as the fallback) strict rect-center containment
    /// (no primary-monitor fallback for off-screen rects); finally
    /// `focused_monitor`. `Focused` is resolved by the caller.
    pub(crate) fn monitor_for_new_window(
        &self,
        win_rect: &Rect,
        policy: crate::config::NewWindowMonitor,
    ) -> MonitorId {
        let use_cursor = policy == crate::config::NewWindowMonitor::Cursor;
        use_cursor
            .then(|| self.monitor_under_cursor())
            .flatten()
            .or_else(|| {
                self.monitors
                    .values()
                    .find(|m| m.contains_rect_center(win_rect))
                    .map(|m| m.id)
            })
            .unwrap_or(self.focused_monitor)
    }

    pub(crate) fn move_focused_column_to_monitor_transactional(
        &mut self,
        target_monitor: MonitorId,
        insert_at_start: bool,
    ) -> std::result::Result<Option<u64>, String> {
        let source_monitor = self.focused_monitor;
        if source_monitor == target_monitor {
            return Ok(None);
        }
        let src_idx = self.active_workspace_idx(source_monitor);
        let tgt_idx = self.active_workspace_idx(target_monitor);
        let mut source = self
            .workspaces
            .get(&source_monitor)
            .and_then(|workspaces| workspaces.get(src_idx))
            .cloned()
            .ok_or_else(|| format!("Source workspace missing for monitor {}", source_monitor))?;
        let mut target = self
            .workspaces
            .get(&target_monitor)
            .and_then(|workspaces| workspaces.get(tgt_idx))
            .cloned()
            .ok_or_else(|| format!("Target workspace missing for monitor {}", target_monitor))?;

        let Some(focused) = source.focused_window() else {
            return Ok(None);
        };
        let source_row = source.focused_row();
        let column_index = source.focused_column_index();
        let column = &source.columns_for_row(source_row)[column_index];
        let moved_windows = column.windows().to_vec();
        let minimized: Vec<_> = column
            .windows()
            .iter()
            .copied()
            .filter(|&window| source.is_minimized(window))
            .collect();
        if column
            .windows()
            .iter()
            .any(|&window| target.contains_window(window))
        {
            return Err("Target workspace already contains a moved window".to_string());
        }
        let new_width =
            self.moved_column_width(focused, source_monitor, src_idx, &[], target_monitor);
        let column = source
            .remove_column_for_transfer(source_row, column_index)
            .ok_or_else(|| "Focused column missing from source workspace".to_string())?;
        let target_row = target.focused_row();
        // Column indices run left to right on every monitor, including RTL
        // ones, so this lands the column at the edge it crossed.
        let index = if insert_at_start {
            0
        } else {
            target.columns_for_row(target_row).len()
        };
        target.insert_column_at_row(column, target_row, index);
        for window in minimized {
            target.mark_minimized(window);
        }
        target
            .focus_window(focused)
            .map_err(|e| format!("Failed to focus moved window: {}", e))?;
        if let Some(new_width) = new_width {
            new_width.apply_to_focused_column(&mut target, self.viewport_width_for(target_monitor));
        }
        target.ensure_focused_visible(self.viewport_width_for(target_monitor));
        source.reconcile_scroll_bounds(self.viewport_width_for(source_monitor));

        self.workspaces.get_mut(&source_monitor).unwrap()[src_idx] = source;
        self.workspaces.get_mut(&target_monitor).unwrap()[tgt_idx] = target;
        self.forget_stashed_windows(&moved_windows);
        self.focused_monitor = target_monitor;
        Ok(Some(focused))
    }

    /// The layout viewport for a monitor: its full work area. Single source of
    /// truth for the rect fed into `compute_placements*`; columns fill the work
    /// area edge to edge with no shared-edge inset.
    pub(crate) fn layout_viewport(&self, monitor_id: MonitorId) -> Rect {
        self.monitors
            .get(&monitor_id)
            .map(|m| match Orientation::of(m.work_area) {
                Orientation::Vertical =>
                    Rect::new(m.work_area.x, m.work_area.y, m.work_area.height, m.work_area.width),
                Orientation::Horizontal => m.work_area,
            })
            .unwrap_or_else(|| Rect::new(0, 0, FALLBACK_VIEWPORT_WIDTH, FALLBACK_VIEWPORT_HEIGHT))
    }

    /// Returns the physical work area and orientation of a monitor.
    /// Falls back to the layout viewport rect and `Horizontal` if the monitor is unknown.
    pub(crate) fn monitor_work_area_and_orient(
        &self,
        monitor_id: MonitorId,
    ) -> (Rect, Orientation) {
        self.monitors
            .get(&monitor_id)
            .map(|m| (m.work_area, Orientation::of(m.work_area)))
            .unwrap_or_else(|| (self.layout_viewport(monitor_id), Orientation::Horizontal))
    }
}

/// Axis orientation of a monitor. Vertical monitors (portrait) swap the
/// layout engine's primary (X) axis onto the physical Y axis.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Orientation {
    Horizontal,
    Vertical,
}

impl Orientation {
    pub(crate) fn of(work_area: Rect) -> Self {
        if work_area.height > work_area.width { Self::Vertical } else { Self::Horizontal }
    }
    pub(crate) fn is_vertical(self) -> bool {
        self == Self::Vertical
    }
    pub(crate) fn primary(self, r: Rect) -> i32 {
        match self { Self::Horizontal => r.x, Self::Vertical => r.y }
    }
    pub(crate) fn primary_size(self, r: Rect) -> i32 {
        match self { Self::Horizontal => r.width, Self::Vertical => r.height }
    }
    pub(crate) fn primary_coord(self, x: i32, y: i32) -> i32 {
        match self { Self::Horizontal => x, Self::Vertical => y }
    }
}

/// Returns the primary-axis length (layout viewport width) for a monitor.
/// For vertical monitors this is `work_area.height`; for horizontal, `work_area.width`.
pub(crate) fn monitor_viewport_width(m: &leopardwm_platform_win32::MonitorInfo) -> i32 {
    Orientation::of(m.work_area).primary_size(m.work_area)
}

/// A monitor's device name (e.g. `\\.\DISPLAY2`), or `""` if it's unknown.
/// Borrows only the monitor map, so callers can still mutate workspaces.
pub(crate) fn monitor_device_name(
    monitors: &HashMap<MonitorId, MonitorInfo>,
    monitor_id: MonitorId,
) -> &str {
    monitors.get(&monitor_id).map_or("", |m| m.device_name.as_str())
}

/// Rotate a layout-space rect to physical screen coordinates for a vertical
/// monitor. The layout engine receives a viewport with swapped width/height,
/// so its "x" axis maps to the monitor's Y axis and vice versa.
pub(crate) fn rotate_layout_rect(layout_rect: Rect, work_area: Rect) -> Rect {
    Rect::new(
        work_area.x + (layout_rect.y - work_area.y),
        work_area.y + (layout_rect.x - work_area.x),
        layout_rect.height,
        layout_rect.width,
    )
}
