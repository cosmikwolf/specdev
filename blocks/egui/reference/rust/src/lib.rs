//! Compiled reference examples for the egui block codegen prose.
//!
//! Type-checks the editor-closure + panel emission worked examples in
//! `../codegen/rust/README.md` against pinned egui + nih_plug_egui. A std
//! host-target lib: it type-checks every call without linking the GUI backend.
//! CI-only; never inlined.
pub mod editor;
pub mod panels;
