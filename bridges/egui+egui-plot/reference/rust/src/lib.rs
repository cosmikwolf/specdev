//! Compiled reference examples for the egui+egui-plot bridge codegen
//! prose. Each module mirrors the SHAPE `/spec-gen` emits for one case.
//! CI-only; never inlined.
//!
//!   `region_plots` — one plot filling one region's body
//!   `stacked`      — several plots in one region, at declared heights
//!                    and on a shared axis
//!
//! Neither endpoint block contributes egui, so this crate stands in for
//! a host: it declares egui directly and re-exports it below, the same
//! crate-root line every real host emits. Emitted code reaches egui
//! through `crate::egui` and plot types through `egui_plot::`.
// A dependency declared but never imported is one the unit contributes to
// user projects with no compiled witness that it is needed or that its
// version is right. `deny` rather than `warn` so the build fails wherever
// it runs -- the reference-crate CI script uses a plain `cargo build`, and
// a warning nobody reads catches nothing.
#![deny(unused_crate_dependencies)]

/// The host's obligation, standing in for `pub use eframe::egui;`.
pub use egui;

pub mod region_plots;
pub mod stacked;
