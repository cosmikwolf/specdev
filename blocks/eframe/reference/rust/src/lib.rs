//! Compiled reference examples for the eframe block codegen prose.
//!
//! Each module mirrors the SHAPE of one emission kind the block documents
//! (app, window, repaint, persistence, viewport) so `cargo build`
//! type-checks the worked examples in `../codegen/rust/README.md` against
//! the pinned eframe 0.36 API. CI-only; never inlined. A std host LIB: it
//! type-checks every example without opening a window or entering an
//! event loop.

pub mod app;
pub mod persistence;
pub mod repaint;
pub mod viewport;
pub mod window;
