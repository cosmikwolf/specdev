# elf-dwarf — implementation guidance

Rules an agent follows when it writes or edits code for a module with an `[elf_dwarf]` table: reading an ELF image
and its DWARF to produce addresses and types. No check verifies them; review does.

## Scope and dependencies

- This block reads a file and produces addresses. No probe, transport or memory-reading crate here.
- Keep the container crate's compression support: an image built with compressed debug sections otherwise loads as
  if it had no debug info.
- The DWARF crate's DIE-access API changed between minor versions. Check accessor shapes against the version in
  `Cargo.toml`, not against older examples.

## Loading

- Loading has two stages: an owner of the section bytes and a borrowing view over them. Keep both in one owner;
  never return the view alone. Prefer the zero-copy reader unless the view must outlive its bytes or cross threads.
- The image path is a runtime input, a parameter.
- A missing debug section yields empty bytes, not an error: debug info is routinely partial.
- `endian = "auto"` reads the image's own byte order.
- `require_linked` checks the container kind first. In an unlinked object every address is a relocation
  placeholder, so the symbol table is uniformly wrong rather than visibly broken.

## Addresses and types

- Evaluate `DW_AT_location` fully, resolving relocated and indexed addresses. Never match only the first address
  operation: DWARF 5 producers use indexed addresses, and the shortcut returns an empty symbol list with no error.
- Any other evaluation request means the variable is not statically allocated: skip it. Accept a single address
  piece only; a value split across locations has no one address.
- Follow `DW_AT_type` through typedef and qualifier tags to a terminal type; classify base types by
  `DW_AT_encoding`. An array's size is element size times its subrange counts (count, or upper bound plus one).
- Read a DIE's children only up to the null entry that closes the list.
- Cap the type-follow depth: malformed or cyclic debug info must not hang the tool.

## Selections

- Resolve `name_pattern` (only `*`) into literal fragments in the code; no glob crate.
- Walk variables only; skip `extern` declarations (no storage in this image); skip function statics unless
  `include_function_statics` is set. A file-scope variable sits at depth one, under the unit root.
- Size from the type's byte size, then the variable's own, then zero. Carry the declaring unit's name on every
  result: two units can each declare a static with one name.
- An empty selection is a normal state, not an error. A type reference of another form degrades to an unclassified
  variable; it is never dropped.
