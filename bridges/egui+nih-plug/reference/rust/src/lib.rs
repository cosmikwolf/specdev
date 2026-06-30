//! Compiled reference examples for the egui+nih-plug bridge codegen prose.
//!
//! Each module mirrors the SHAPE of the per-widget stanza the bridge emits
//! inside a `parameter_controls` region (a render fn taking the editor-closure
//! `ui` + `setter` + the captured `Arc<Params>`), so `cargo build` type-checks
//! the worked examples in `../codegen/rust/README.md` against the pinned
//! nih-plug + egui API. A widget call against a non-existent type or wrong
//! method fails this build. CI-only; never inlined.
//!
//! It is a `std` host-target LIB (not embedded, not a bin): the lib type-checks
//! every widget construction without linking the GUI backend or running a
//! plugin, so no `impl Plugin`, editor shell, or export macros are needed.
//!
//! Covers the widget kinds whose documented emission type-checks against the
//! pinned `nih_plug_egui` (`slider`, `button`, `numeric_field`). The remaining
//! kinds documented in the README type table reference APIs absent from this
//! pin and are tracked as prose-correctness gaps, not shipped here.

pub mod params;

pub mod button;
pub mod numeric_field;
pub mod slider;
