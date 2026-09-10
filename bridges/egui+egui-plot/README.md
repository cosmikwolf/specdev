# egui+egui-plot bridge

Pairwise wiring between the `egui` block (interface topology) and the
`egui-plot` block (plot topology). Contributes region ↔ plot bindings
under `bindings."egui+egui-plot".region_plots[]`.

## When to use this bridge

Inline `egui+egui-plot` when:

- Your module has both an `[egui]` table (panels and regions) and an
  `[egui_plot]` table (plots and series).
- You want the surrounding layout and the charts specified together,
  rather than a region whose body says "hand-authored" and a plot that
  nothing places.
- You stack several plots in one region on a shared time axis.

Don't inline if:

- Your plots live in a hand-rolled `Ui` with no declared region
  structure. The `egui-plot` block works alone; a plot draws into any
  `Ui`.
- Your interface has regions but no plots, or plots but no declared
  interface topology. Each block stands alone.

## What this bridge contributes

**Placement.** The `egui` block describes regions and is deliberately
silent about what fills a region's body; the `egui-plot` block describes
plots and is deliberately silent about where a plot is drawn. Neither
can express the placement without naming the other's types, which is
precisely the leg-3 violation the bridge primitive exists to absorb.

The binding also carries the two sizing decisions the pairing creates:
`fill` (take the region's space, or keep the plot's declared height) and
`order` (draw order within a region holding several plots).

One relation, `region_draws_plot`, from `egui.Region` to
`egui-plot.Plot`. `REDUCE_UNRESOLVED` flags a binding naming a region or
plot that was never declared.

This bridge is the direct analogue of `egui+nih-plug`, which binds
regions to parameters. The same block, the same region kinds, a
different thing being placed inside them.

## What this bridge does NOT contribute

- **A new crate.** There is no join product. A region body already has a
  `&mut Ui` and `Plot::show` already takes one, so they compose
  directly. The bridge therefore ships no `deps.yaml`.
- **egui.** Neither endpoint block contributes it either. It arrives
  from the host — the `eframe` block for a native application, or the
  project itself for another integration.
- **The data.** A series says how it is drawn, not where its samples
  come from.
- **The region's layout.** `ui.vertical` / `ui.horizontal` / a grid is
  the region's own declaration. The binding only decides what goes
  inside.

## Inline order

```
/spec-inline egui
/spec-inline egui-plot
/spec-inline-bridge egui+egui-plot
```

Both blocks must be inlined first: the bridge's reduction references
`egui.Region` and `egui-plot.Plot`, and an unresolved endpoint kind
surfaces as a silent `REDUCE_UNRESOLVED` rather than a clear error.

A **host** is required transitively, since neither block contributes
egui. Inline `eframe` for a native application.

## Co-residency

A binding's `region` and `plot` must both be declared on the same module
that carries the bindings table. Cross-module wiring is not supported in
v1.

## Sharp-test rationale

**Concrete pairing.** Both endpoints name specific published crates,
egui and egui_plot, and the pairing is one specific version pair —
egui_plot 0.37 against egui 0.36.

**Codegen-complete.** Conforming TOML produces working code: a plot
emitted into a region's body with the region's own layout around it, a
stack of plots at declared heights on a linked axis, and the conditional
builder rebinding that axis linking requires. Both worked examples are
compiled by `reference/rust/`.

**The contribution belongs to the bridge.** Placement cannot live in
`egui` without teaching it about plot types, and cannot live in
`egui-plot` without teaching it about regions. Either would break the
other block's leg 3 — which is exactly the argument the `egui+nih-plug`
bridge makes for parameter binding.

## Deferred

- **Series-to-source binding.** Placing a plot is one half; filling its
  series from a data producer is the other. That is a different pairing
  and would be a different bridge, against whichever unit owns the
  samples.
- **Region-relative sizing.** `fill` offers "all of it" or "the plot's
  own height". A fraction of the region is a plausible third value,
  deferred until a layout needs it.
- **Interaction routing.** A plot's `PlotResponse` carries the hovered
  item and the pointer coordinate, which a surrounding region might want
  for a readout. Modelling that needs a vocabulary for where the value
  goes, and is deferred until a project asks.
