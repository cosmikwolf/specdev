//! Compiled reference companion crate for the flutter-rust-bridge block. CI-only.
//!
//! `api` is the module the FRB generator scans (`rust_input: crate::api`);
//! `frb_generated` is the generator's own output, committed at the pinned
//! codegen version so this crate compiles without re-running the generator.
// A dependency declared but never imported is one the unit contributes to user
// projects with no compiled witness that it is needed or that its version is
// right. `deny` rather than `warn` so the build fails wherever it runs -- the
// reference-crate CI script uses a plain `cargo build`, and a warning nobody
// reads catches nothing. If a dep is genuinely used only under a cfg, add
// `use <crate> as _;` here rather than downgrading this.
#![deny(unused_crate_dependencies)]
pub mod api;
mod frb_generated;
