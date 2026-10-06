# egui-plot — implementation guidance

Rules an agent follows when it writes or edits code for a module with an `[egui_plot]` table. No check verifies
them; review does.

## Dependencies

- This block adds egui_plot only. egui comes from the host's crate-root re-export; never add egui (or its math
  crate) here — a second egui fails at the seam.
- egui_plot's version must pair with the host's egui version. A mismatch puts two eguis in the graph.

## Plots

- One function per plot, taking a `Ui`, with the samples it draws passed in as parameters: where samples come from
  is not this block's concern.
- A field the spec decides (style, marker, colour, axes) is a literal in the code, not a runtime string match.
- The plot's id source is the spec's `id`, verbatim and stable: egui_plot keeps pan and zoom state under it, so a
  rename resets the user's view. It is never drawn.
- `legend = "none"` sets no legend. Each `Axes` field is a two-axis mask; an axis is locked by leaving it out of the
  mask.
- `link_group` names the group for linked axes (with `link_axes` as the mask) and, when `link_cursor` is true, for
  the linked cursor. Two unrelated plots never share a group id.
- `follow_window` pins the x bounds inside the plot's build closure every frame, and takes x off auto-bounds whatever
  `auto_bounds` says. Do not express it as fixed default bounds: a zero-width window before the first sample
  panics.

## Series

- Each `[[egui_plot.series]]` entry is one item added inside its plot's build closure.
- The label is the item's identity and goes into the constructor, not a later rename; two series with one label
  share a legend entry. With no `label`, use the `id`.
- Give every series that can be empty a stable colour — the spec's, else one derived from its `id`. An empty item is
  skipped, and automatic colours shift as items come and go.
- A live plot fed from a ring buffer stores the plot's own point type, so the samples can be borrowed each frame
  without a copy.
