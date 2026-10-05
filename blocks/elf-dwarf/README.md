# elf-dwarf block

Debug-info block targeting [object](https://github.com/gimli-rs/object)
0.40 for the ELF container and [gimli](https://github.com/gimli-rs/gimli)
0.34 for the DWARF inside it. Contributes a namespaced `[elf_dwarf]`
table on `Module` describing **symbol recovery**: which images are read,
and which statically-allocated variables are selected out of them.

The block answers one question — "what global variables does this image
have, at what addresses, of what type and size?" — and stops there.

## When to use this block

Inline `elf-dwarf` when your project:

- Needs the address of a named variable in a compiled firmware image, so
  that something else can read it.
- Presents a symbol picker built from an image the user selects, rather
  than from a hard-coded list.
- Wants to reload symbols after a rebuild, because addresses move
  whenever the firmware is recompiled.
- Needs the *type* as well as the address: a `u32`, an `i16` and an
  `f32` are all four-or-fewer bytes at an address, and only the debug
  info says which.

Don't use this block if:

- Your target's symbols are known and stable enough to hard-code. A
  constant is simpler than a DWARF walk.
- You only need a symbol table, not types. ELF's `.symtab` gives names,
  addresses and sizes without touching DWARF, and is far cheaper.
- Your image is not ELF. Mach-O and PE also carry DWARF and `object`
  reads them, but the linked-image and section-name assumptions here are
  written for ELF.

## What this block contributes

| Kind | Table | Models |
|---|---|---|
| `Image` | `[[elf_dwarf.images]]` | One ELF: endianness policy, reload policy, linked-image and function-static rules |
| `Selection` | `[[elf_dwarf.selections]]` | A named filter over one image's globals: name glob, accepted type kinds, size ceiling |

One intra-block relation, `selection_in_image`, makes a selection naming
an undeclared image fail at validate time.

### A selection is a filter, not a list

The obvious alternative is to declare the variables of interest by name.
That is the wrong shape: variable names are a property of the firmware
being observed, not of the tool observing it. They change with every
refactor, and a spec listing them goes stale on someone else's commit.
A pattern survives a rebuild; a list does not.

## What this block does NOT contribute

- **The file path.** Which build is being observed is a property of the
  session, not the spec. Every loader takes the path.
- **Reading the target.** The block produces an address, a width and a
  type classification. Fetching the bytes at that address is a separate
  concern and belongs in a bridge — see `elf-dwarf+probe-rs`.
- **Display or storage.** No UI, no file format, no plotting.
- **Writing DWARF.** Read-only; gimli's writer is not pulled in.

## Sharp-test rationale

**Leg 1 — concreteness.** The block names two specific published crates
reading one specific documented format. The vocabulary — `DwarfSections`,
`EntriesCursor`, `EvaluationResult`, `DW_ATE_*` — is gimli's own. A
GDB-driven or `nm`-driven symbol source would be a different block,
because it would model the same information with an entirely different
surface.

**Leg 2 — rule-pack completeness.** Every field constrains the code:
the image's endianness and link requirement, each selection's pattern,
kinds and scope. The guidance carries the rules no check can verify —
the full location evaluation (both `DW_OP_addr` and `DW_OP_addrx`), the
bounded type walk, array sizes from subranges.

**Leg 3 — no cross-block reach.** The guidance names no other
block's types. The output is a plain struct of name, address, size and
kind — the narrowest seam available, and one that reaches toward nothing.

## Companion units

| Companion | Bridge | What the pairing adds |
|---|---|---|
| `probe-rs` | `elf-dwarf+probe-rs` | Turns a recovered address, width and type into a sampled read against a running target, so a viewer can watch symbols rather than raw addresses |

## Known sharp edges

Both were found by compiling a test crate while the block was written, and
both are in the guidance:

- **`DW_OP_addrx`.** GCC emits `DW_OP_addr`; clang and rustc emitting
  DWARF 5 emit `DW_OP_addrx` into `.debug_addr`. Code that matches only
  the first operation returns *nothing* for the latter, silently. The
  guidance requires the full evaluation path for exactly this reason.
- **`object::File<'data, R>`.** The lifetime and the reader type must be
  the same lifetime. Elided separately they do not unify, and the error
  points at the signature rather than the cause.

## Deferred

- **Cross-unit type references.** `DW_AT_type` can arrive as
  `DebugInfoRef` when a linker merges units. v1 handles `UnitRef` and
  degrades the rest to unclassified rather than dropping them.
- **Struct member walking.** Recovering the offset and type of a field
  inside a struct is the natural next step, and needs a path vocabulary
  (`config.pid.kp`) plus member traversal. Deferred until a project
  wants sub-field selection.
- **The `.symtab` fast path.** For names and addresses without types,
  the ELF symbol table is far cheaper than a DWARF walk. Worth a
  `source = "symtab" | "dwarf"` field once a project has an image whose
  DWARF walk is too slow.
- **Line tables.** gimli reads them and they would support "which source
  line declared this", which is a source-navigation feature rather than
  a symbol-recovery one.
- **Non-ELF containers.** `object` reads Mach-O and PE; the block's
  linked-image check and section handling assume ELF.
