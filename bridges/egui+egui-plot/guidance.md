# egui+egui-plot — implementation guidance

Rules an agent follows when it writes or edits a plot placed in an egui region. No check verifies them yet; review
does. EGP-001..004 are rules a future unit check will verify.

## Placement

- This pairing adds no crate: a plot draws into the region's `Ui`. egui comes from the host's re-export.
- A bound region's body is the plot. The wrapper comes from the region's own layout; add no extra container.
- `fill = "available"` lets the plot take the region's space; `fill = "plot_height"` uses the plot's `height`.

## Stacks

- A region bound to several plots draws them by `order`, ascending, then the unordered ones in declaration order.
- Linking calls are conditional, so the plot builder is rebound, not chained.

## Rules

- **EGP-001:** the bound region's purpose is `free_form`; other purposes have their own bodies.
- **EGP-002:** in a multi-plot region every binding uses `fill = "plot_height"` and every plot has a `height` —
  otherwise the first plot takes all the space and the rest vanish. In a scroll area a plot with no height grows
  without limit.
- **EGP-003:** a plot is bound into one region only; two copies with one id fight over one stored view.
- **EGP-004:** plots stay in step through their own `link_group`, whatever regions hold them; no grouping derived
  from regions.
