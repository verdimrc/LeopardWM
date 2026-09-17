# Changelog — haha branch

## [Unreleased] — diff between branch `haha` (`c65df72`) and `main` (`ed0aa13`)

### Features

- New windows open on the monitor under the cursor.

- `behavior.notify_elevation_blocked` silences the toast when a privilege-elevated window is excluded from tiling.

- RTL layout mode: columns accumulate from the right edge of the viewport instead of the left. Enable with `[layout] rtl_monitor_indices = [N]` where N is the Windows display number (e.g. `\\.\DISPLAY12` → `12`).

- Overview improvements:
  - Shows on all monitors simultaneously with Ctrl+Alt+Win+Space.
  - Secondary monitor overlays are now clickable and highlight on hover.
  - Shows the monitor device name and empty workspace rows on monitors with no windows.
  - Panels and monitor labels are right-aligned on RTL monitors.

- Window rules support `tile_on_os_monitor` to respect the OS-chosen monitor.
  Useful for PowerPoint presentation mode: respects where PowerPoint places both the slide show and the presenter view windows, while still tiling the presenter view.

- Right Ctrl and Right Alt are interchangeable with their left counterparts in hotkeys.
  Opt in with `behavior.symmetric_modifiers = true` in the config. Useful for one-handed operation.

### Fixes

- Focused monitor placement is restored correctly for new windows.

- Windows that self-resize during the suppression window are snapped back.

- A vacated column stays visible on the source monitor after moving a window away.

- Wrong window targeted by commands after layout animation completes (briefly tested; no consistent repro trigger).

- Window resizing after opening a new window now targets the focused window.

- Clicking a window in another monitor's overview no longer focuses the wrong window.
