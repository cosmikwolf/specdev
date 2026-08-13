//! Compiled reference examples for the eframe block codegen prose.
//!
//! Each module mirrors the SHAPE of one emission kind the block documents
//! (app, window, repaint, persistence, viewport) so `cargo build`
//! type-checks the worked examples in `../codegen/rust/README.md` against
//! the pinned eframe 0.36 API. CI-only; never inlined. A std host LIB: it
//! type-checks every example without opening a window or entering an
//! event loop.
// A dependency declared but never imported is one the unit contributes to user
// projects with no compiled witness that it is needed or that its version is
// right. `deny` rather than `warn` so the build fails wherever it runs -- the
// reference-crate CI script uses a plain `cargo build`, and a warning nobody
// reads catches nothing. If a dep is genuinely used only under a cfg, add
// `use <crate> as _;` here rather than downgrading this.
#![deny(unused_crate_dependencies)]

/// This block's host obligation for the `egui` block: one crate-root
/// re-export, so emitted topology code resolves `crate::egui`.
pub use eframe::egui;

pub mod app;
pub mod host_alias;
pub mod persistence;
pub mod repaint;
pub mod viewport;
pub mod window;
