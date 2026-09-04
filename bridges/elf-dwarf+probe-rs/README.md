# elf-dwarf+probe-rs bridge

Pairwise wiring between the `elf-dwarf` block (symbol recovery from
debug info) and the `probe-rs` block (memory sampling over a debug
probe). Contributes selection ↔ sampler bindings under
`bindings."elf-dwarf+probe-rs".sampled_selections[]`.

This is the bridge that turns a raw-address memory reader into a
**variable** viewer.

## When to use this bridge

Inline `elf-dwarf+probe-rs` when:

- Your module has both an `[elf_dwarf]` table (images and selections)
  and a `[probe_rs]` table (a session and at least one sampler).
- You want a sampler to read *named symbols* rather than addresses
  hand-copied from a map file.
- Your firmware gets rebuilt while the tool is running, and the tool has
  to keep pointing at the right addresses afterwards.

Don't inline if:

- Your addresses are fixed and known — a memory-mapped peripheral block,
  or a linker-pinned buffer at a constant address. The probe-rs block
  alone covers that, and a DWARF walk buys nothing.
- You read symbols but never touch a target (a static analysis tool), or
  read a target but never resolve symbols (a peripheral poker). Each
  block stands alone; the bridge is for the pairing.

## What this bridge contributes

**The type-to-width mapping.** This is the load-bearing part, and it is
genuinely unownable by either endpoint:

- The `elf-dwarf` block knows a variable is a 2-byte signed integer at
  `0x2000_0410`. It knows nothing about reading memory.
- The `probe-rs` block knows how to read 2 bytes from `0x2000_0410`. It
  knows nothing about what they mean, and its address list is
  deliberately a runtime input.
- Only the pairing knows that the first should *choose* the second — and
  that the recovered signedness has to survive all the way to the point
  where bytes become a number, or `-1i16` is displayed as 65535.

The binding table also carries three policies the pairing needs:
`width_from` (trust the type, or the sampler's uniform width),
`aggregates` (skip structs and arrays, or read them as bytes), and
`on_missing` (an empty selection is normal, or fatal).

One relation, `selection_sampled_by`, from `elf-dwarf.Selection` to
`probe-rs.Sampler`. `REDUCE_UNRESOLVED` then flags a binding naming a
selection or sampler that was never declared.

## What this bridge does NOT contribute

- **A new crate.** There is no join product. probe-rs already accepts a
  `u64` and elf-dwarf already produces one; the pairing needs code, not
  an adapter. This bridge therefore ships no `deps.yaml`.
- **Cross-module wiring.** Both endpoints must be declared on the module
  carrying the bindings table (`binding_locality: endpoint_module`).
- **Build-identity checking.** Nothing here can verify that the ELF and
  the running firmware are the same build. See EDPR-003 in the codegen
  prose — the failure mode is plausible values from wrong addresses, and
  the bridge documents it rather than pretending to prevent it.
- **Display.** Sampled values reach a UI as ordinary data.

## Inline order

```
/spec-inline elf-dwarf
/spec-inline probe-rs
/spec-inline-bridge elf-dwarf+probe-rs
```

Both blocks must be inlined first: the bridge's reduction references
`elf-dwarf.Selection` and `probe-rs.Sampler`, and an unresolved endpoint
kind surfaces as a silent `REDUCE_UNRESOLVED` rather than a clear error.

## Co-residency

A binding's `selection` and `sampler` must both be declared on the same
module that carries the bindings table. In practice that module is the
project's acquisition module — it owns the debug session, so it owns
both sides. Cross-module wiring is not supported in v1.

## Sharp-test rationale

**Concrete pairing.** Both endpoints name specific libraries: gimli plus
object on one side, probe-rs on the other. This is not "symbol source
meets memory reader" in the abstract.

**Codegen-complete.** Conforming TOML produces working code: a plan
built from real DWARF classifications, reads dispatched at the right
widths, sign extension and float bit-casting on the way back, and a
wholesale re-plan on image reload. Both worked examples are compiled by
`reference/rust/`, against both blocks' pinned APIs.

**The contribution belongs to the bridge.** The width mapping cannot go
in `elf-dwarf` without teaching it about probe access widths, and cannot
go in `probe-rs` without teaching it about DWARF encodings. Either would
break the other block's leg 3.

## Deferred

- **Struct field selection.** Sampling `config.pid.kp` rather than all of
  `config` needs member traversal in the elf-dwarf block first; the
  bridge would then bind a field path instead of a whole variable.
- **Write-back.** probe-rs writes memory as readily as it reads it, and
  a tuning tool wants to set a variable, not only watch it. Deferred
  because a write needs a type-aware encode path — the mirror of the
  interpret path — plus a policy about what may be written at all.
- **Bandwidth planning.** A plan knows every read's width, so it could
  estimate whether a sampler's `interval_ms` is achievable and warn when
  it is not. Deferred until there is real evidence of what the transport
  sustains.
- **Address-range coalescing.** Adjacent variables could be fetched in
  one block read. Worth doing only once there is a measurement showing
  per-read overhead dominates.
