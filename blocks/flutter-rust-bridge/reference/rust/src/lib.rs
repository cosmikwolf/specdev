//! Compiled reference companion crate for the flutter-rust-bridge block. CI-only.
//!
//! `api` is the module the FRB generator scans (`rust_input: crate::api`);
//! `frb_generated` is the generator's own output, committed at the pinned
//! codegen version so this crate compiles without re-running the generator.
pub mod api;
mod frb_generated;
