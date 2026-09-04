//! Compiled reference examples for the egui-plot block codegen prose.
//!
//! Type-checks the plot + series emission worked examples in
//! `../codegen/rust/README.md`. A std host-target lib: it type-checks
//! every call without linking a rendering backend. CI-only; never
//! inlined.
//!
//! The block contributes `egui_plot` but not egui, so this crate stands
//! in for a host: it declares egui directly and re-exports it below,
//! which is the same crate-root line every real host emits
//! (`pub use eframe::egui;`). Every module below then reaches egui
//! through `crate::egui`, exactly as emitted code does, and reaches plot
//! types through `egui_plot::` directly.
// A dependency declared but never imported is one the unit contributes to
// user projects with no compiled witness that it is needed or that its
// version is right. `deny` rather than `warn` so the build fails wherever
// it runs -- the reference-crate CI script uses a plain `cargo build`, and
// a warning nobody reads catches nothing.
#![deny(unused_crate_dependencies)]

/// The host's obligation, standing in for `pub use eframe::egui;`.
/// Emitted code refers to `crate::egui`.
pub use egui;

pub mod plots;
pub mod series;
