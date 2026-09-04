# elf-dwarf — Rust codegen conventions

These conventions describe how an `elf_dwarf` block table on a module
spec maps to Rust source. They layer on top of the project's profile's
`module.conventions.md`; concerns specific to reading DWARF (section
loading, DIE traversal, location evaluation, type resolution) live here.

The block targets [object](https://github.com/gimli-rs/object) for the
ELF container and [gimli](https://github.com/gimli-rs/gimli) for the
DWARF inside it.

## Crate dependencies

This block contributes two crates:

```toml
gimli = "0.34"
object = { version = "0.40", features = ["read", "std", "compression"] }
```

Keep `compression`: `CompressedData::decompress` only handles zlib and
zstd behind that feature, so an image built with
`--compress-debug-sections` fails to load without it, and the failure
looks like missing debug info rather than a missing feature.

The gimli pin is load-bearing. gimli reshaped its DIE-access API within
the 0.3x series, and the changes are of the kind that older examples get
silently wrong (see below).

**Do NOT** add a probe, transport, or memory-reading crate from this
block. It reads a file and produces addresses; fetching anything from
those addresses is a separate concern.

## Type emission

Emit one loader type per image and one filter function per selection, in
the module named by the spec, following the project's profile's
`module.conventions.md`.

**The two gimli values must be emitted as a pair.** Loading is two-stage:
an owning value holds the section bytes and a borrowing value indexes
into them.

```rust
let sections: gimli::DwarfSections<Cow<'_, [u8]>> = gimli::DwarfSections::load(load_section)?;
let dwarf: gimli::Dwarf<gimli::EndianSlice<'_, gimli::RunTimeEndian>> =
    sections.borrow(|section| gimli::EndianSlice::new(section, endian));
```

`Dwarf<R>` takes one parameter; `EndianSlice<'input, Endian>` takes a
lifetime and an endianity type. The `Dwarf` borrows from `sections`, so
emit both into the same owner and never return the `Dwarf` alone.

**Do NOT** emit `Dwarf::borrow` — it is deprecated in favour of
`DwarfSections::borrow`. **Do NOT** reach for `EndianReader` unless the
`Dwarf` genuinely must outlive its bytes or be `Send`; `EndianSlice` is
zero-copy and `Copy`, and is the right default.

## Image emission

One `[[elf_dwarf.images]]` entry becomes one load path. The path itself
is a **runtime input** — emit it as a parameter.

`object::File<'data, R = &'data [u8]>` takes a lifetime and a reader
type. The two must be the **same** lifetime; elided separately they do
not unify and the signature is rejected:

```rust
pub fn parse<'a>(bytes: &'a [u8]) -> Result<object::File<'a, &'a [u8]>, object::Error> {
    object::File::parse(bytes)
}
```

`object::Object` and `object::ObjectSection` are **traits** and must be
in scope, or `section_by_name` and `uncompressed_data` do not resolve.

The section loader hands gimli each debug section by id:

```rust
let load_section = |id: gimli::SectionId| -> Result<Cow<'_, [u8]>, gimli::Error> {
    Ok(match object_file.section_by_name(id.name()) {
        Some(section) => section
            .uncompressed_data()
            .unwrap_or(Cow::Borrowed(&[][..])),
        None => Cow::Borrowed(&[][..]),
    })
};
```

A **missing section must return empty bytes, not an error**. Debug info
is routinely partial — an image with no `.debug_addr` or no line tables
is still perfectly readable — and failing the load rejects images that
work fine.

`endian = "auto"` reads `object_file.is_little_endian()`. `require_linked`
checks the container kind up front:

```rust
matches!(
    object_file.kind(),
    object::ObjectKind::Executable | object::ObjectKind::Dynamic
)
```

Iterating units uses an **inherent** `next` returning
`Result<Option<gimli::UnitHeader<R>>, gimli::Error>`, not a `std`
iterator:

```rust
let mut units = dwarf.units();
while let Some(header) = units.next()? {
    let unit: gimli::Unit<R> = dwarf.unit(header)?;
}
```

**Do NOT** write `for header in dwarf.units()` — it does not compile.
**Do NOT** skip `require_linked` when the spec asks for it: in an
unlinked object file every address is an unresolved relocation
placeholder, normally zero, so the symbol table is uniformly wrong
rather than visibly broken.

## Address and type resolution

**Emit the `Evaluation` path for `DW_AT_location`, never the
first-operation shortcut.** This is the most consequential rule here.
Matching the first `Operation::Address` reads only `DW_OP_addr`, which is
what GCC emits. Clang and rustc emitting DWARF 5 use `DW_OP_addrx`, an
index into `.debug_addr`, and the shortcut silently returns nothing for
every global in those images — an empty symbol list with no error.

```rust
let mut evaluation: gimli::Evaluation<R> = expression.evaluation(unit.encoding());
let mut result = evaluation.evaluate()?;

loop {
    match result {
        gimli::EvaluationResult::Complete => break,
        gimli::EvaluationResult::RequiresRelocatedAddress(address) => {
            result = evaluation.resume_with_relocated_address(address)?;
        }
        gimli::EvaluationResult::RequiresIndexedAddress { index, relocate: _ } => {
            let address = dwarf.address(unit, index)?;
            result = evaluation.resume_with_indexed_address(address)?;
        }
        _ => return Ok(None),
    }
}

let pieces: Vec<gimli::Piece<R>> = evaluation.result();
```

Any other `Requires*` means the variable is not statically allocated — a
local in a register or on the stack — and returning `None` correctly
skips it. `evaluation.result()` **panics** if the evaluation did not
complete, which the loop guarantees. Accept only a single
`Piece { location: Location::Address { address }, .. }`; more than one
piece means a value split across locations, which one address cannot
describe.

Following `DW_AT_type` matches `AttributeValue::UnitRef(offset)`, then
loops through `DW_TAG_typedef`, `const_type`, `volatile_type`,
`restrict_type` and `atomic_type` until something terminal. At
`DW_TAG_base_type`, read `DW_AT_encoding` and classify:

```rust
gimli::DW_ATE_boolean => ScalarKind::Bool,
gimli::DW_ATE_float => ScalarKind::Float,
gimli::DW_ATE_signed | gimli::DW_ATE_signed_char => ScalarKind::Signed,
gimli::DW_ATE_unsigned | gimli::DW_ATE_unsigned_char => ScalarKind::Unsigned,
```

**An array has no `DW_AT_byte_size`.** Its size is the element size times
the product of its `DW_TAG_subrange_type` children's counts
(`DW_AT_count`, or `DW_AT_upper_bound + 1`). Reading children uses
`entries_at_offset` plus `next_entry`, which stops at the null DIE
closing the child list — `next_dfs` would walk past it into the
following siblings.

**Do NOT** leave the type walk unbounded. Cap the follow depth;
malformed or cyclic debug info must not hang a tool that is only
populating a symbol list.

### gimli 0.34 accessor shapes

Older examples get these wrong, and the compiler error is confusing:

- `entry.attr_value(name)` returns `Option<AttributeValue<R>>` — **not**
  `Result`. Do not write `entry.attr_value(x)?`.
- `entry.attr(name)` returns `Option<&Attribute<R>>` — **not** `Result`.
- `entry.attrs()` returns a **slice**, not a fallible iterator.
- `cursor.next_dfs()` returns `Result<Option<&DIE>, gimli::Error>`,
  **not** `Result<Option<(isize, &DIE)>, gimli::Error>`. Read the depth
  from `entry.depth()`.
- `EntriesCursor<'abbrev, R>` has **one** lifetime, not two.
- `Unit` has inherent `entries()` / `entry(offset)` taking **no**
  abbreviations argument, which shadow the `UnitHeader` methods that do.
  `unit.entries(&unit.abbreviations)` fails to compile.

## Selection emission

One `[[elf_dwarf.selections]]` entry becomes one filtered walk. Resolve
`name_pattern` into literal fragments **at codegen time** — the pattern
vocabulary is `*` only, so a glob crate would be a dependency earning
nothing.

The walk skips three things, each for its own reason:

```rust
if entry.tag() != gimli::DW_TAG_variable {
    continue;
}

// An `extern` declaration: no storage in this image.
if matches!(
    entry.attr_value(gimli::DW_AT_declaration),
    Some(gimli::AttributeValue::Flag(true))
) {
    continue;
}

if !filter.include_function_statics && entry.depth() > 1 {
    continue;
}
```

Prefer the **type's** byte size, falling back to the variable's own
`DW_AT_byte_size` and then to zero. Carry the declaring unit name on
every result: two compilation units can each declare a `static` of the
same name, and the unit is the only thing telling them apart.

**Do NOT** filter on `entry.depth() == 0` — a file-scope variable sits at
depth 1, under the compilation-unit root, so testing for zero yields
nothing at all. **Do NOT** treat an empty selection as an error; a
pattern matching nothing is a normal state for a build that has not
declared those symbols yet. **Do NOT** assume `DW_AT_type` is always
`UnitRef` — a linker that merges units can emit `DebugInfoRef`, and
while v1 handles only `UnitRef`, the fallthrough must degrade to an
unclassified variable rather than dropping it.
