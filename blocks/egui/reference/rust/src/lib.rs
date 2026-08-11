//! Compiled reference examples for the egui block codegen prose.
//!
//! Type-checks the editor-closure + panel emission worked examples in
//! `../codegen/rust/README.md` against pinned egui + nih_plug_egui. A std
//! host-target lib: it type-checks every call without linking the GUI backend.
//! CI-only; never inlined.
// A dependency declared but never imported is one the unit contributes to user
// projects with no compiled witness that it is needed or that its version is
// right. Plugin CI builds reference crates with `-D warnings`.
#![warn(unused_crate_dependencies)]
pub mod editor;
pub mod panels;
