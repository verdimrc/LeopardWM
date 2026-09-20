# Changelog — haha branch

## [Unreleased] — diff between `ed0aa13` .. HEAD

### Features

- New windows open on the monitor under the cursor.
- `behavior.notify_elevation_blocked` silences the toast when a privilege-elevated window is excluded from tiling.
- RTL layout mode: columns accumulate from the right edge of the viewport instead of the left, and new
  columns open to the left of the focused column. Enable with `[layout] rtl_monitor_indices = [N]` where N
  is the Windows display number (e.g. `\\.\DISPLAY2` → `2`).
- Overview improvements:
  - Shows on all monitors simultaneously with Ctrl+Alt+Win+Space.
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
- Right Ctrl and Right Alt are interchangeable with their left counterparts in hotkeys.
  Opt in with `behavior.symmetric_modifiers = true` in the config. Useful for one-handed operation.

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
