# Changelog — haha branch

## [Unreleased] — diff between `ed0aa13` .. HEAD

### Features

- New windows open on the monitor under the cursor.
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
  - Secondary monitor overlays are now clickable and highlight on hover.
  - Shows the monitor device name and empty workspace rows on monitors with no windows.
  - Panels and monitor labels are right-aligned on RTL monitors.
- Window rules support `tile_on_os_monitor` to respect the OS-chosen monitor.
  Useful for PowerPoint presentation mode: respects where PowerPoint places both the slide show and the
  presenter view windows, while still tiling the presenter view.
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

### New config fields

| Field | Type | Default | Description |
|---|---|---|---|
| `layout.rtl_monitor_indices` | `[u32]` | `[]` | Monitor display numbers that use RTL layout. |
| `layout.desktop_peek_min_width` | `f64` | `0.25` | Minimum ghost column width as a fraction of monitor width. |
| `behavior.notify_elevation_blocked` | `bool` | `true` | Show a toast when a privilege-elevated window is excluded from tiling. Set to `false` to silence it. |
| `behavior.symmetric_modifiers` | `bool` | `false` | Treat Right Ctrl / Right Alt as interchangeable with their left counterparts in hotkeys. |
| `hotkeys.toggle_overview_all` | `string` | `"Ctrl+Alt+Win+Space"` | Toggle overview on all monitors simultaneously. |
| `hotkeys.toggle_desktop_peek_anchored` | `string` | `"Ctrl+Alt+'"` | Reveal desktop; focused window stays in place. |
| `hotkeys.toggle_desktop_peek` | `string` | `"Ctrl+Alt+Shift+'"` | Reveal desktop; focused window moves to the minimum-width mark. |
| Window rule: `tile_on_os_monitor` | `bool` | `false` | Tile the window on whichever monitor the OS places it, instead of the focused monitor. |

### Fixes

- Focused monitor placement is restored correctly for new windows. (The implementation is retained to
  support reverting to pre-0.2.8 behavior, but is superseded in practice by "new windows open on the
  monitor under the cursor".)
- Apps that shrink themselves after being tiled no longer stay smaller than their column until the next
  layout re-apply.
- A vacated column stays visible on the source monitor after moving a window away.
- Wrong window targeted by commands after layout animation completes (briefly tested; no consistent repro
  trigger).
- Window resizing after opening a new window now targets the focused window.
