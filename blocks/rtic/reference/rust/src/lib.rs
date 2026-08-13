//! Compiled reference for the rtic block codegen prose.
//!
//! Type-checks every worked example in `../codegen/rust/README.md` against
//! the pinned rtic API. CI-only; never inlined.
//!
//! A dependency declared but never imported would be a crate the block
//! contributes to user projects without exercising, so make that fail here.
#![warn(unused_crate_dependencies)]
#![no_std]

pub mod app;
