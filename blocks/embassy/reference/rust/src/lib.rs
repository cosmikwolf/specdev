//! Compiled reference examples for the embassy block codegen prose.
//!
//! Each module mirrors the SHAPE of one emission kind the block documents
//! (task, channel) so `cargo build --target thumbv8m...` type-checks the worked
//! examples in `../codegen/rust/README.md` against the pinned embassy API.
//! CI-only; never inlined. A no_std LIB: it type-checks every example without
//! linking an executor or a runner.
// A dependency declared but never imported is one the unit contributes to user
// projects with no compiled witness that it is needed or that its version is
// right. Plugin CI builds reference crates with `-D warnings`.
#![warn(unused_crate_dependencies)]
#![no_std]

pub mod channel;
pub mod task;
