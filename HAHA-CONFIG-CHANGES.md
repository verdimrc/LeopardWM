# Config changes after merging main into haha

Main replaced the centering keys. The old keys still work, but update `config.toml` to the new
ones.

## 1. Centering keys

| Old | New |
|---|---|
| `centering_mode = "center_columns_when_narrower"` | `center_columns = "fit"` and `centering_mode = "just_in_view"` |
| `center_single_column = true` | `center_columns = "single"` |
| `center_single_column = false` | `center_columns = "off"` |

This applies to `[layout]` and to each `[[monitor_rules]]` entry. If a section has both old keys,
`"center_columns_when_narrower"` wins, as before.

`full_screen_tolerance_px` now applies to `center_columns = "fit"`.

## 2. Your config today

Path: `%APPDATA%\leopardwm\config\config.toml`.

```toml
[layout]
center_columns = "fit"           # was: center_single_column = true
centering_mode = "just_in_view"  # was: "center_columns_when_narrower"

[[monitor_rules]]
display = 2
center_columns = "off"           # was: center_single_column = false
centering_mode = "just_in_view"

[[monitor_rules]]
display = 3
full_screen_tolerance_px = 6     # unchanged
```

## 3. Removed keys

`layout.rtl_monitor_indices` and `layout.default_width_preset_monitor_overrides` were already
ignored. Main now warns about unknown keys, so delete them if they are still in the file.
