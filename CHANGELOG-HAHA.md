# Changelog — haha branch

## [Unreleased] — diff between `c673836` .. HEAD

### Features

- New windows can open on the monitor under the mouse cursor: `behavior.new_window_monitor = "cursor"`,
  or Settings → Behavior → New window monitor → Monitor under cursor. The default stays `"opening"`.
- `behavior.notify_elevation_blocked` silences the toast when a privilege-elevated window is excluded from tiling.
- Right Ctrl and Right Alt are interchangeable with their left counterparts in hotkeys.
  Opt in with `behavior.symmetric_modifiers = true` in the config. Useful for one-handed operation.
- RTL layout mode: columns accumulate from the right edge of the viewport instead of the left, and new
  columns open to the left of the focused column. Enable with `[layout] rtl_monitor_indices = [N]` where N
  is the Windows display number (e.g. `\\.\DISPLAY2` → `2`).

  The motivating setup: a wide external monitor sitting to the left of a center main display. LTR places
  new windows at the far left of that monitor — far from the center display, forcing a wide eye sweep.
  RTL clusters windows toward the right edge of the external monitor, right next to the center display,
  so your eyes barely move between the two screens.
- Overview improvements:
  - Shows on all monitors simultaneously with Ctrl+Alt+Win+Space (`hotkeys.toggle_overview_all`).
  - Shows the monitor device name
  - Shows empty workspace rows on monitors with no windows.
- Window rules support `new_window_monitor` (`"opening"`, `"focused"` or `"cursor"`) to override
  `behavior.new_window_monitor` per app. E.g. `"opening"` for PowerPoint presentation mode: respects
  where PowerPoint places both the slide show and the presenter view windows, while still tiling the
  presenter view.
- Vertical monitor support:
  - Automatically detected from portrait orientation (height > width) — no configuration required; rotate
    the monitor in Windows Display Settings.
  - The layout engine's horizontal axis maps to the monitor's vertical axis — columns become rows, and
    left/right navigation moves up/down. Drag-and-drop, column resizing, and cross-monitor operations all
    respect the rotated axis.
  - Overview workspace panels are arranged side by side as tall columns instead of horizontal stripes.
    Window cards are shown in their physical (portrait) orientation, stacked top to bottom.
- Desktop peek: reveals the left portion of the Windows desktop without closing any windows. The
  motivating use case is drag-and-drop to the desktop — Windows places desktop icons on the left, but a
  tiling layout covers that area.
  - `Ctrl+Alt+'` (`hotkeys.toggle_desktop_peek_anchored`) slides a ghost column in from the left and
    keeps the focused window in its current position. The ghost fills the space to its left (at least
    `layout.desktop_peek_min_width`, default 25%). Exception: if the window is already at the left edge,
    it shifts right by the minimum width to make room.
  - `Ctrl+Alt+Shift+'` (`hotkeys.toggle_desktop_peek`) always moves the focused window to the
    minimum-width mark, regardless of where it started.
  - The ghost area is click-through; the desktop beneath remains interactive.
  - Peek is monitor-local — operations on other monitors leave it intact. Any window operation on the
    peeked monitor (focus change, move, close, minimize, maximize, fullscreen) exits peek and restores
    the previous scroll position.
  - Only available on LTR horizontal monitors.
- Window rules: `column_width` now accepts a 1-based preset index (e.g. `column_width = 2`,
  same convention as `layout.default_width_preset`) or a per-display map
  (e.g. `column_width = { "1" = 0.5, "2" = 2 }`) in addition to the existing viewport
  fraction. Map keys are quoted Windows display indices (`"1"` = `\\.\DISPLAY1`). Useful
  when the same app needs a different initial width on each monitor.
- `behavior.new_width_on_monitor_move` sets the width a tiled window gets when it becomes a new column
  on another monitor, whether moved with the hotkey or dragged (Shift+drag, or a drag onto a monitor
  with no windows):
  - `"upstream"` (default): main's behavior. The window gets the target monitor's default width
    (Shift+drag keeps the column's width). It does not use the window rule's `column_width`.
  - `"same_proportion_as_source_monitor"`: the same share of the screen as on the source monitor.
  - `"window_rule_width_or_default_width"`: the width it would get if opened there: the window rule's
    `column_width`, else the default width.

  A window dragged into an existing column takes that column's width, in every mode.
- `layout.centering_mode = "center_columns_when_narrower"` centers all columns together when they are
  narrower than the screen. When they are wider, they start at the left edge (right edge with RTL) and
  the screen scrolls like `"just_in_view"`.
- `layout.default_width_preset_monitor_overrides` lets each monitor open new columns at its
  own width preset (e.g. `default_width_preset_monitor_overrides = { "2" = 3 }`). Keys are
  Windows display numbers; monitors not listed use `layout.default_width_preset`.
- New-window placement: `Ctrl+Alt+Shift+N` (`hotkeys.toggle_new_window_placement_once`)
  activates a one-shot override to the opposite of the current default placement for the
  next window only. When active, pressing it again cancels the override instead.
  - Explicitly changing the persistent default (tray menu or `Ctrl+Alt+N`) or reloading
    the config always cancels a pending override.
- Tray icon shows a badge reflecting new-window placement:

  | Badge | Meaning |
  |---|---|
  | None | New windows always open in a new column |
  | Amber | Only the next window opens in-column |
  | Blue | New windows always open in-column |
  | Green | Only the next window opens in a new column |

### New config fields

| Field | Type | Default | Description |
|---|---|---|---|
| `layout.rtl_monitor_indices` | `[u32]` | `[]` | Monitor display numbers that use RTL layout. ⚠️ [Known issue with GUI settings](#known-issues-with-gui-settings) |
| `layout.desktop_peek_min_width` | `f64` | `0.25` | Minimum ghost column width as a fraction of monitor width. ⚠️ [Known issue with GUI settings](#known-issues-with-gui-settings) |
| `behavior.notify_elevation_blocked` | `bool` | `true` | Show a toast when a privilege-elevated window is excluded from tiling. Set to `false` to silence it. ⚠️ [Known issue with GUI settings](#known-issues-with-gui-settings) |
| `behavior.symmetric_modifiers` | `bool` | `false` | Treat Right Ctrl / Right Alt as interchangeable with their left counterparts in hotkeys. ⚠️ [Known issue with GUI settings](#known-issues-with-gui-settings) |
| `hotkeys.toggle_overview_all` | `string` | `"Ctrl+Alt+Win+Space"` | Toggle overview on all monitors simultaneously. |
| `hotkeys.toggle_desktop_peek_anchored` | `string` | `"Ctrl+Alt+'"` | Reveal desktop; focused window stays in place. |
| `hotkeys.toggle_desktop_peek` | `string` | `"Ctrl+Alt+Shift+'"` | Reveal desktop; focused window moves to the minimum-width mark. |
| Window rule: `new_window_monitor` | `"opening" \| "focused" \| "cursor"` | (none) | Overrides `behavior.new_window_monitor` for matching windows. |
| `layout.default_width_preset_monitor_overrides` | `{ "N" = u32 }` | `{}` | Per-display-index override of `layout.default_width_preset`. ⚠️ [Known issue with GUI settings](#known-issues-with-gui-settings) |
| Window rule: `column_width` (extended) | `f64 \| u32 \| { "N" = f64\|u32 }` | (none) | Now accepts a preset index or a per-display-index map in addition to a viewport fraction. ⚠️ [Known issue with GUI settings](#known-issues-with-gui-settings) |
| `hotkeys.toggle_new_window_placement_once` | `string` | `"Ctrl+Alt+Shift+N"` | Activate a one-shot new-window-placement override for the next window only. |
| `behavior.new_window_monitor` (extended) | `"opening" \| "focused" \| "cursor"` | `"opening"` | Adds `"cursor"`: open new windows on the monitor under the mouse cursor. |
| `behavior.new_width_on_monitor_move` | `"upstream" \| "same_proportion_as_source_monitor" \| "window_rule_width_or_default_width"` | `"upstream"` | Width of a tiled window moved to another monitor. ⚠️ [Known issue with GUI settings](#known-issues-with-gui-settings) |
| `layout.centering_mode` (extended) | `"center" \| "just_in_view" \| "on_overflow" \| "center_columns_when_narrower"` | `"center"` | Adds `"center_columns_when_narrower"`: center all columns together when they are narrower than the screen. ⚠️ [Known issue with GUI settings](#known-issues-with-gui-settings) |

### Fixes

- Closing the last window on a workspace no longer switches away from it.
- Tray-menu and hotkey/IPC-triggered config changes no longer silently overwrite `config.toml`.
- A vacated column stays visible on the source monitor after moving a window away.
- Wrong window targeted by commands after layout animation completes (briefly tested; no consistent repro
  trigger).
- Window resizing after opening a new window now targets the focused window.
- After Ctrl+Alt+C centers a window with `center_past_edges = true`, moving focus to a window that is
  already fully visible no longer scrolls the screen back to the edge.
- Cross-monitor window moves:
  - Fixed: after moving a window to a new monitor, monitor-focus shortcuts must also focus the new monitor.
  - Fixed apps never joining the tiled layout when they don't report their initial window position.
  - Fixed incorrect sizing when moving to a monitor with different DPI scaling.

  Note: main's `ed2f51f` and `00e9b66` fix DPI rescaling on display-settings changes and floating/monitor-removal
  edge cases; these are different bugs than the ones above.

### Known issues with GUI settings

Saving from the settings window rewrites `config.toml` from the fields the window shows. Fields it
doesn't show are reset or removed:

| Field | After saving from the settings window |
|---|---|
| `layout.rtl_monitor_indices` | Reset to `[]`. |
| `layout.desktop_peek_min_width` | Reset to `0.25`. |
| `behavior.notify_elevation_blocked` | Reset to `true`. |
| `behavior.symmetric_modifiers` | Reset to `false`. |
| `behavior.new_width_on_monitor_move` | Reset to `"upstream"`. |
| `layout.centering_mode = "center_columns_when_narrower"` | Kept, but the settings window shows "Center" and the tray menu checks no centering mode. Picking another mode in either replaces it. |
| `layout.default_width_preset_monitor_overrides` | Removed. |
| Window rule: `column_width` (extended) | Fractions and preset index `1` are kept; preset index `2` or higher and per-display maps are removed. |
