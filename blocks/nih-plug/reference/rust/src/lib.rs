//! Compiled reference examples for the nih-plug block codegen prose.
//!
//! Type-checks the parameter-struct emission + per-kind range constructors in
//! `../codegen/rust/README.md` against pinned nih-plug. A std host-target lib:
//! it type-checks parameter construction without running a plugin. CI-only.
pub mod params;
pub mod sysex;
