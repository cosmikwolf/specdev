//! Compiled reference examples for the elf-dwarf block codegen prose.
//!
//! Type-checks the worked examples in `../codegen/rust/README.md` against
//! the pinned gimli and object APIs. A std host-target lib. CI-only;
//! never inlined.
//!
//! Each module mirrors one part of the `[elf_dwarf]` table:
//!   `images`     — loading an ELF's debug sections and building a `Dwarf`
//!   `selections` — walking the DIE tree and filtering global variables
//!   `types`      — resolving a static address and a `DW_AT_type` chain
//!
//! The generic-arity annotations here are the point of the file: gimli
//! reshaped its DIE-access API within the 0.3x series, and several of
//! these signatures differ from what older code and older documentation
//! show.
// A dependency declared but never imported is one the unit contributes to
// user projects with no compiled witness that it is needed or that its
// version is right. `deny` rather than `warn` so the build fails wherever
// it runs -- the reference-crate CI script uses a plain `cargo build`, and
// a warning nobody reads catches nothing.
#![deny(unused_crate_dependencies)]

pub mod images;
pub mod selections;
pub mod types;
