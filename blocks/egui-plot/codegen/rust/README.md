# egui-plot — Rust codegen conventions

These conventions describe how an `egui_plot` block table on a module
spec maps to Rust source. They layer on top of the project's profile's
`module.conventions.md`; concerns specific to egui_plot (plot
construction, axis masks, bounds policy, item constructors) live here.

The block targets [egui_plot](https://github.com/emilk/egui_plot) as its
concrete library.

## Crate dependencies

This block contributes one crate:

```toml
egui_plot = "0.37"
```

It does **not** contribute egui. egui arrives from whichever unit
supplies the crate-root `pub use ...::egui;` alias, and emitted code
reaches it through `crate::egui`. Plot types come from `egui_plot::`
directly.

The version pairing is load-bearing: egui_plot 0.37 builds against egui
0.36, and a host supplying a different minor version puts two eguis in
the graph whose `Ui` types do not unify. That failure appears at the
seam, which is the right place for it, but it is worth recognising on
sight.

**Do NOT** add egui to this block's dependencies to "make it
self-contained" — that is precisely what creates the two-egui failure.
**Do NOT** add `emath` for `Vec2b`; it comes from `crate::egui`.

## Type emission

Emit one function per plot, taking `&mut crate::egui::Ui`, in the module
named by the spec and following the project's profile's
`module.conventions.md`. Series are emitted **inside** their plot's build
closure, not as free functions — a plot item only exists in the context
of a `PlotUi`.

Where the samples come from is **not modelled by this block**. Emit each
plot function with the data it draws as a parameter.

Every enum-valued schema field resolves at codegen time into a literal:
`style` into a `LineStyle`, `marker` into a `MarkerShape`, `color` into
`egui::Color32::from_rgb(r, g, b)` with the hex digits already split.

**Do NOT** emit a runtime string match for a field the spec has already
decided. The examples below show matches only because a reference crate
must exercise every variant; generated code takes the one branch.

## Plot emission

One `[[egui_plot.plots]]` entry becomes one `Plot`. Chain the builder in
the order the fields are listed here so diffs stay readable:

```rust
use crate::egui;
use egui_plot::{Corner, Legend, Plot, PlotResponse, PlotUi};

let legend = Legend::default().position(Corner::RightTop);

let _response: PlotResponse<()> = Plot::new("signals")
    .legend(legend)
    .x_axis_label("t [s]")
    .y_axis_label("value")
    .height(240.0)
    .allow_drag(egui::Vec2b::new(true, false))
    .allow_zoom(egui::Vec2b::new(true, false))
    .allow_scroll(egui::Vec2b::new(true, false))
    .auto_bounds(egui::Vec2b::new(false, true))
    .link_axis("scope_time", egui::Vec2b::new(true, false))
    .link_cursor("scope_time", egui::Vec2b::new(true, false))
    .show(ui, |plot_ui: &mut PlotUi<'_>| {
        plot_ui.set_plot_bounds_x((now - window)..=now);
    });
```

`Plot::new` takes an **id source, not a title** — it is never drawn.
egui_plot persists pan and zoom state under it, so emit the spec `id`
verbatim and keep it stable: renaming a plot silently resets whatever
view the user had set up.

`legend = "none"` emits **no** `.legend(...)` call at all. The other
values map to the matching `Corner` variant.

Every `Axes` field becomes a `Vec2b` mask — `x` is
`Vec2b::new(true, false)`, `y` is `Vec2b::new(false, true)`, `both` is
both true, `none` is both false. There is **no** axis-lock method: an
axis is locked by leaving it out of the mask.

`link_group` supplies the group id to `.link_axis(group, mask)` with
`link_axes` as the mask, and to `.link_cursor(group, mask)` when
`link_cursor` is true. Emit neither call when there is no group.

`follow_window` becomes a `set_plot_bounds_x` call **inside** the build
closure, evaluated every frame. Because pinning the x bounds necessarily
takes x off auto-bounds, emit the `auto_bounds` mask with x cleared
whenever a follow window is present, regardless of what the spec's
`auto_bounds` said about x.

**Do NOT** emit `.default_x_bounds(min, max)` from a follow window — it
debug-asserts `min < max` and panics on a zero-width window, which is
exactly the state a viewer is in before its first sample. **Do NOT**
call `set_plot_bounds_x` outside the closure; there is no `PlotUi` there.
**Do NOT** reuse one `link_group` id across unrelated plots: a plot
belongs to at most one group, and sharing a group couples their axes
permanently.

## Series emission

One `[[egui_plot.series]]` entry becomes one plot item added inside the
build closure of the plot it names.

**Every item constructor takes its label as the first argument.** This is
the single easiest thing to get wrong, because the builder-style
`.name(...)` also exists:

```rust
use egui_plot::{Bar, BarChart, HLine, Line, LineStyle, MarkerShape, Points, VLine};

plot_ui.line(
    Line::new(label.to_owned(), samples)
        .color(rgb(0x4c, 0xaf, 0x50))
        .width(1.5)
        .style(LineStyle::Solid),
);

plot_ui.points(
    Points::new(label.to_owned(), samples)
        .shape(MarkerShape::Circle)
        .radius(3.0)
        .filled(true)
        .color(rgb(0x21, 0x96, 0xf3)),
);

plot_ui.hline(HLine::new(label.to_owned(), y).width(1.0).style(LineStyle::Solid));
plot_ui.vline(VLine::new(label.to_owned(), x).style(LineStyle::dashed_dense()));
```

`bar_chart` is the one exception worth noting: `Bar::new` takes position
and height only, with **no** label, because the chart carries the name.

```rust
let bars: Vec<Bar> = buckets
    .iter()
    .map(|&(argument, height)| Bar::new(argument, height))
    .collect();

plot_ui.bar_chart(BarChart::new(label.to_owned(), bars).color(rgb(0x67, 0x3a, 0xb7)));
```

The label is also the item's **identity**: the constructor derives the
item id from it, so two series sharing a label share one legend entry
and toggle together. When the spec gives no `label`, emit the `id`.

`style` maps to `LineStyle::Solid` or one of the four constructors
(`dashed_dense`, `dashed_loose`, `dotted_dense`, `dotted_loose`).
`marker` maps to the matching `MarkerShape` variant.

Samples may be owned or borrowed, and the choice has a real consequence.
Owned takes `Vec<[f64; 2]>`. Borrowed requires `&[PlotPoint]` — there is
**no** conversion from `&[[f64; 2]]` — and the lifetime is not elidable:

```rust
pub fn line_borrowed<'a>(plot_ui: &mut PlotUi<'a>, label: &str, samples: &'a [PlotPoint]) {
    plot_ui.line(Line::new(label.to_owned(), samples).width(1.5));
}
```

`PlotUi<'a>` is invariant behind a mutable reference, so the samples must
live exactly as long as the plot's own lifetime parameter and both must
be named. A ring buffer feeding a live plot should therefore be stored
as `Vec<PlotPoint>`; storing `Vec<[f64; 2]>` forces a clone every frame.

**Do NOT** omit `color` for a series that can be empty. An item with no
data is skipped entirely — it does not even appear in the legend — and
automatic colour assignment counts items in the order they are added, so
colours shift as series come and go. A spec that gives a colour is
asking to avoid exactly this; a spec that does not still deserves a
stable colour derived from the series `id` rather than left to the
counter. **Do NOT** emit `.name(...)` as the primary label — it changes
the displayed text without changing the id, which is a deliberate tool
for a value that updates per frame, not the way to name a series.
