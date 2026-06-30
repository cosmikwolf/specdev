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
//! Covers all 8 widget kinds (WIDGET-KIND-V1). Three map to APIs that exist
//! directly (`slider`/`button`/`numeric_field`); five were corrected to the
//! pinned `nih_plug_egui` reality (no `ParamButton`/`ParamComboBox`/`Knob`;
//! `Enum::variants()` yields names; `Param::unit()` not `unit_string()`).

pub mod params;

pub mod button;
pub mod dropdown;
pub mod knob;
pub mod numeric_field;
pub mod radio_group;
pub mod slider;
pub mod toggle;
pub mod value_display;
