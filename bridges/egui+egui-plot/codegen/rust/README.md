# egui+egui-plot — Rust codegen conventions

These conventions describe how a `bindings."egui+egui-plot"` table on a
module spec maps to Rust source. They layer on top of the project's
profile's `module.conventions.md`, and on top of both endpoint blocks'
own codegen prose.

**Bridge prose wins.** Where this file describes a bound region's body,
it replaces the `egui` block's instruction to emit a `free_form` region
as a TODO marker with a hand-authored body: under this binding, the body
is the plot.

## Crate dependencies

This bridge contributes **no new crate**. A plot draws into whatever
`Ui` it is handed, and a region body already has one, so the two compose
at that `&mut Ui` with no adapter. Both blocks' deps.yaml supply
everything the emitted code needs.

egui itself still arrives from the host — neither endpoint block
contributes it. Emitted code reaches it through `crate::egui` and plot
types through `egui_plot::`.

## Placement emission

The region scaffold comes from the interface topology; the binding fills
its body:

```rust
use crate::egui;
use egui_plot::{Corner, Legend, Plot, PlotResponse, PlotUi};

ui.vertical(|ui| {
    let _response: PlotResponse<()> = Plot::new("signals")
        .legend(Legend::default().position(Corner::RightTop))
        .x_axis_label("t [s]")
        .y_axis_label("value")
        .allow_drag(egui::Vec2b::new(true, false))
        .allow_zoom(egui::Vec2b::new(true, false))
        .auto_bounds(egui::Vec2b::new(false, true))
        .show(ui, |_plot_ui: &mut PlotUi<'_>| {});
});
```

The wrapper (`ui.vertical`, `ui.horizontal`, a grid) is chosen by the
**region's own layout**, not by this binding. The binding decides only
what goes inside it.

`fill = "available"` emits **no** `.height(...)`: the plot takes the
region's space. `fill = "plot_height"` emits `.height(...)` from the
plot's own `height` field.

**Do NOT** emit the region's hand-authored-body TODO marker for a bound
region — the binding is what fills it. **Do NOT** wrap the plot in an
extra layout container the region did not ask for; the region's layout
is already emitted around it.

## Stacking emission

A region bound to more than one plot emits them in `order`, ascending.
Entries without an order come after those with one, in declaration
order.

```rust
ui.vertical(|ui| {
    for bound in plots {
        let mut plot = Plot::new(bound.id)
            .height(bound.height)
            .legend(Legend::default())
            .allow_drag(egui::Vec2b::new(true, false))
            .auto_bounds(egui::Vec2b::new(false, true));

        if let Some(group) = bound.link_group {
            plot = plot
                .link_axis(group, egui::Vec2b::new(true, false))
                .link_cursor(group, egui::Vec2b::new(true, false));
        }

        let _response: PlotResponse<()> = plot.show(ui, |plot_ui: &mut PlotUi<'_>| {
            plot_ui.set_plot_bounds_x((now - window)..=now);
        });
    }
});
```

Note the builder is rebound (`plot = plot.link_axis(...)`) rather than
chained, because the linking calls are conditional. Every `Plot` builder
method consumes and returns `self`.

**Every plot in a stack must carry an explicit height** — see EGP-002.
With `fill = "available"` the first plot consumes the region and the
rest render into nothing, which shows up as a blank area rather than as
an error.

Inside a scroll area the same rule is sharper still: a scroll area gives
its content unbounded height, so a plot with no explicit height expands
without limit.

## Discipline rules

Cross-cutting constraints no single block can check, and that the
reduction cannot express as relations. Emitted code enforces them.

- **EGP-001 — region purpose.** The bound region's `purpose` must be
  `free_form`. The other purposes each emit their own body — a metering
  region emits read-side boilerplate, a label block emits `ui.label`
  calls — and a plot drawn into one of those either collides with that
  body or silently replaces it. Reject at codegen time with a message
  naming the region and its purpose.
- **EGP-002 — height in a stack.** A region bound to more than one plot
  requires `fill = "plot_height"` on every binding for that region, and
  a `height` on every plot so bound. Reject the combination of
  `fill = "available"` with a multi-plot region rather than emitting a
  layout where all but the first plot vanish.
- **EGP-003 — one region per plot.** A plot bound into two regions is
  drawn twice with the same id, and egui_plot persists pan and zoom
  state under that id — so the two copies fight over one stored view and
  neither behaves. Reject; declare a second plot instead.
- **EGP-004 — link groups are cross-plot, not cross-region.** Plots in a
  shared `link_group` stay in step regardless of which regions hold
  them. Do not emit any region-derived grouping; the group id from the
  plot's own field is the whole mechanism.
