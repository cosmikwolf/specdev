# elf-dwarf+probe-rs — implementation guidance

Rules an agent follows when it writes or edits code that samples, through a debug probe, the variables that DWARF
debug info locates. No check verifies them; review does.

## The handoff

- No crate is added: the probe reads a `u64` address and the debug-info reader produces one. Under this binding the
  sampler's addresses come from the selection, not from a runtime parameter.
- Three stages, kept apart: recover (run the selection over the image), plan (map each variable to an access),
  read (run the plan against the core). The first two depend on the image, the third on the target.

## The plan

- Each access records a width and an interpretation together: one four-byte read is a different number as an
  unsigned integer or a float.
- Booleans read one byte; floats read four or eight; integers read one, two, four or eight. Any other size is
  refused, never rounded up.
- `width_from = "sampler_default"` takes the sampler's width but keeps the type's interpretation.
- `aggregates`: `skip` drops non-scalar variables; `bytes` reads each as one block read, never as several words.
- Sign-extend a signed value before widening it; decode a float only at its own size.
- A refused variable is surfaced, never dropped silently.
- `on_missing = "error"` is checked when planning: an empty plan is then an error.

## Reading

- Every read is an ordinary probe read, under the sampler's `max_batch` and `on_error`. Never re-run the selection
  inside the sampling loop.

## Reload

- With `reload = "watch"` on the image, a reload stops sampling, rebuilds the plan from scratch, and restarts. Never
  patch the old plan: a variable can change size or vanish, and a name does not keep its type.

## Rules

- **EDPR-001:** a selection that admits only non-scalar kinds, bound with `aggregates = "skip"`, always yields
  nothing — refuse it.
- **EDPR-002:** one selection may feed several samplers, but each plans on its own; no shared plan.
- **EDPR-003:** the image and the target must be the same build. Where the target exposes a build id, check it;
  otherwise state the assumption in the documentation.
