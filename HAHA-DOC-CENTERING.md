# Centering options

`centering_mode` decides **when** the screen scrolls as you move focus. `center_past_edges` decides
whether centering may leave **empty space** next to the first or last column.

| You want | Setting |
|---|---|
| The focused window always in the middle, screen moves a lot | `"center"` |
| The screen moves as little as possible | `"just_in_view"` |
| Like `"just_in_view"`, but a window wider than the screen is shown from its middle | `"on_overflow"` |
| All columns centered together while they're narrower than the screen, otherwise like `"just_in_view"` | `"center_columns_when_narrower"` |
| With `"center"`: the first column in the middle too, even if that leaves empty space | add `center_past_edges = true` |

## 1. Examples

Every picture below uses the same setup: a screen 1000px wide (the `[ ]` brackets), with four 400px
columns A B C D. One character is 50px, and `·` is empty space.

```
whole strip:   AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDD
```

## 2. `centering_mode`

This setting decides when the screen scrolls as you move focus. The following examples assume `center_past_edges = false` (default); section 3 shows how
`center_past_edges = true` further impacts window placement.

### 2.1. `center` (default)

This setting always puts the focused column in the middle, as far as possible.

```
focus A   [AAAAAAAABBBBBBBBCCCC]CCCCDDDDDDDD     A can't go to the middle without empty space → stays at the left edge
focus B   AA[AAAAAABBBBBBBBCCCCCC]CCDDDDDDDD     B in the middle
focus C   AAAAAAAABB[BBBBBBCCCCCCCCDDDDDD]DD     C in the middle
focus D   AAAAAAAABBBB[BBBBCCCCCCCCDDDDDDDD]     D stays at the right edge, same reason as A
```

The screen moves on every focus change, so your eyes always look at the middle.

### 2.2. `just_in_view`

This setting scrolls the screen only when the focused column is cut off, and only as far as
needed.

```
focus A   [AAAAAAAABBBBBBBBCCCC]CCCCDDDDDDDD
focus B   [AAAAAAAABBBBBBBBCCCC]CCCCDDDDDDDD     B already fully visible → no scroll
focus C   AAAA[AAAABBBBBBBBCCCCCCCC]DDDDDDDD     C was cut off → scroll until C touches the right edge
focus D   AAAAAAAABBBB[BBBBCCCCCCCCDDDDDDDD]     same, D touches the right edge
then back:
focus C   AAAAAAAABBBB[BBBBCCCCCCCCDDDDDDDD]     C visible → no scroll
focus B   AAAAAAAA[BBBBBBBBCCCCCCCCDDDD]DDDD     B was cut off → B touches the left edge
```

The screen stays still as much as possible, but the focused window can be anywhere on screen.

### 2.3. `on_overflow`

This setting behaves the same as `"just_in_view"`, except for a column wider than the screen.

All four columns above fit on the screen, so `"on_overflow"` gives exactly the same pictures as
`"just_in_view"`. The difference shows only with a window wider than the screen, such as W, 1400px
wide on the 1000px screen:

```
just_in_view   W's left 400px hidden, right edge lined up with the screen's right edge
on_overflow    W centered: 200px hidden on each side
```

### 2.4. `center_columns_when_narrower`

This setting centers all columns together, as one strip, while they're narrower than the screen.
With only A and B open:

```
focus A   [··AAAAAAAABBBBBBBB··]
focus B   [··AAAAAAAABBBBBBBB··]     no scroll: the strip stays centered
```

Once the columns are wider than the screen, as with all four, it behaves like `"just_in_view"`.

## 3. `center_past_edges`

When set to `true`, centering may leave empty space before the first column or after the last one:

```
center_past_edges = true, centering_mode = "center":
focus A   [······AAAAAAAABBBBBB]BBCCCCCCCCDDDDDDDD     A really in the middle, empty space on its left
```

Limitations:

1. **Only effective when `centering_mode = "center"`, or with the "center column" hotkey
   (Ctrl+Alt+C).** When `centering_mode` is `"just_in_view"`, `"on_overflow"`, or
   `"center_columns_when_narrower"`, moving focus never centers a column that fits on the screen,
   so `center_past_edges` makes no practical difference.
2. **The last column is not centered when you focus it.** Focusing D still gives
   `AAAAAAAABBBB[BBBBCCCCCCCCDDDDDDDD]`, i.e., D still at the right edge. You need
   Ctrl+Alt+C to center D.

   NOTE: this looks like a bug, because the setting's description says "the first/last column
   will be truly centered" (last check: commit `fde4289`).

## 4. Related: `center_single_column`

When a workspace has only one window and it's narrower than the screen, this setting centers it
instead of putting it at the left edge. It works with every `centering_mode` and is separate from
`center_past_edges`:

```
off   [AAAAAAAA············]
on    [······AAAAAAAA······]
```
