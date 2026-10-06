# egui-plot block

Plot-topology block targeting
[egui_plot](https://github.com/emilk/egui_plot) 0.37. Contributes a
namespaced `[egui_plot]` table on `Module` describing **plots and the
series drawn inside them** — the charting counterpart to the `egui`
block's panels and regions.

## When to use this block

Inline `egui-plot` when your project:

- Draws charts in an egui interface with a known structure — named
  plots, known series, known appearance.
- Streams data onto a live time axis. The `follow_window` field exists
  for exactly this, and getting a scrolling window right by hand is
  fiddlier than it looks.
- Keeps several plots aligned on a shared axis, such as a signal plot
  above an event timeline.

Don't use this block if:

- Your charts are one-off and hand-drawn. A single plot with an ad-hoc
  closure is lighter than a spec table.
- Your plotting library is something else (plotters, charming). The
  vocabulary here — `Vec2b` axis masks, `PlotUi`, `LineStyle` — is
  egui_plot's own.
- You need a chart type v1 does not model. Box plots, heatmaps, images
  and arrows all exist upstream; see Deferred.

## What this block contributes

| Kind | Table | Models |
|---|---|---|
| `Plot` | `[[egui_plot.plots]]` | Axes and labels, legend corner, height, interaction masks, bounds policy, axis linking |
| `Series` | `[[egui_plot.series]]` | Item kind (line / points / h_line / v_line / bar_chart), label, colour, stroke and marker appearance |

One intra-block relation, `series_in_plot`, makes a series naming an
undeclared plot fail at validate time rather than drawing nothing. This
mirrors the `egui` block's `region_in_panel`.

## What this block does NOT contribute

- **egui.** The block contributes `egui_plot` only. egui arrives from
  whichever unit supplies the crate-root `pub use ...::egui;` alias —
  the `eframe` block for a native application, the `egui+nih-plug`
  bridge for a plugin editor, or the project itself for any other
  integration. The code refers to `crate::egui`.
- **The data.** A series names a channel and says how it is drawn.
  Filling it is the code's job; binding it to a producer is a
  bridge's job.
- **Where the plot sits in the interface.** A plot draws into whatever
  `Ui` it is handed. Placing it inside a surrounding panel or region is
  a bridge concern — see `egui+egui-plot`.

### The version pairing is load-bearing

egui_plot 0.37 depends on egui ^0.36. A host supplying a different minor
version puts two independently-versioned eguis in the dependency graph,
whose `Ui` and `Context` types will not unify. The failure surfaces at
the call seam rather than as a version complaint, so it is worth
recognising on sight: if `plot_ui.line(...)` reports a `Ui` type
mismatch, check the host's egui version before anything else.

## Sharp-test rationale

**Leg 1 — concreteness.** The block names egui_plot, one specific
published crate. It is not a "charting" abstraction: `Vec2b` axis masks,
`PlotUi`, `MarkerShape` and the name-first item constructors are all
egui_plot's own vocabulary, and a different plotting library would model
the same concepts with different types.

**Leg 2 — rule-pack completeness.** Every field constrains the code:
the plot's id, axes masks, bounds policy, link group and follow window;
each series' kind, style, colour and label. The guidance carries the
rules no check can verify (stable ids, label identity, stable colours).

**Leg 3 — no cross-block reach.** The guidance names no other block's
types. It refers to `crate::egui`, which is a crate-root convention any
host satisfies, not a reach into a particular unit.

## Companion units

| Companion | Bridge | What the pairing adds |
|---|---|---|
| `egui` | `egui+egui-plot` | Places a declared plot inside a declared interface region, so the surrounding layout and the chart are specified together |

A host block (`eframe`, or the `egui+nih-plug` bridge) is required
transitively, because something must supply egui. This block does not
care which.

## Deferred

- **More item kinds.** `BoxPlot`, `Heatmap`, `PlotImage`, `Arrows`,
  `Polygon`, `Text` and `Span` all exist upstream. v1 ships the five
  that a signal viewer needs; the others land when a project wants one.
- **Custom axis formatters.** `AxisHints` supports per-tick formatting
  closures, which is how a time axis gets `mm:ss` labels. Modelling a
  closure in TOML needs a vocabulary of named formats rather than
  arbitrary code, and is deferred until that vocabulary is earned.
- **Explicit bounds.** v1 offers `follow_window` and `auto_bounds`. A
  fixed `[min, max]` per axis is a plausible v1.1 field; it was left out
  because the upstream builder debug-asserts a non-empty range, which
  needs care to model safely.
- **Per-series visibility.** Legend click-to-hide is built in and needs
  no spec surface. A spec-declared default-hidden series is deferred.
