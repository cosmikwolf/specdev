//! Compiled reference examples for the nih-plug block codegen prose.
//!
//! Type-checks the parameter-struct emission + per-kind range constructors in
//! `../codegen/rust/README.md` against pinned nih-plug. A std host-target lib:
//! it type-checks parameter construction without running a plugin. CI-only.
// A dependency declared but never imported is one the unit contributes to user
// projects with no compiled witness that it is needed or that its version is
// right. `deny` rather than `warn` so the build fails wherever it runs -- the
// reference-crate CI script uses a plain `cargo build`, and a warning nobody
// reads catches nothing. If a dep is genuinely used only under a cfg, add
// `use <crate> as _;` here rather than downgrading this.
#![deny(unused_crate_dependencies)]
pub mod midi;
pub mod params;
pub mod sysex;
