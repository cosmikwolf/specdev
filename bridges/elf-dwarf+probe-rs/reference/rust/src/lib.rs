//! Compiled reference examples for the elf-dwarf+probe-rs bridge codegen
//! prose. Each module mirrors the SHAPE `/spec-gen` emits for one case.
//! CI-only; never inlined.
//!
//!   `widths`             — the DWARF type to probe access-width mapping,
//!                          which is the decision neither block can make
//!   `sampled_selections` — the whole handoff: recover, plan, read
//!
//! The endpoint types are annotated explicitly throughout, because that
//! is what makes the compiler check them.
// A dependency declared but never imported is one the unit contributes to
// user projects with no compiled witness that it is needed or that its
// version is right. `deny` rather than `warn` so the build fails wherever
// it runs -- the reference-crate CI script uses a plain `cargo build`, and
// a warning nobody reads catches nothing.
#![deny(unused_crate_dependencies)]

// `object` is used only through the loading path that the elf-dwarf block's
// own reference already compiles; this bridge's cases start from a Dwarf
// that is already built. The import keeps the declared dep honest.
use object as _;

pub mod sampled_selections;
pub mod widths;
