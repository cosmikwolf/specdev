//! Compiled reference examples for the nih-plug block codegen prose.
//!
//! Type-checks the parameter-struct emission + per-kind range constructors in
//! `../codegen/rust/README.md` against pinned nih-plug. A std host-target lib:
//! it type-checks parameter construction without running a plugin. CI-only.
// A dependency declared but never imported is one the unit contributes to user
// projects with no compiled witness that it is needed or that its version is
// right. Plugin CI builds reference crates with `-D warnings`.
#![warn(unused_crate_dependencies)]
pub mod midi;
pub mod params;
pub mod sysex;
