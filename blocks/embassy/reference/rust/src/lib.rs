//! Compiled reference examples for the embassy block codegen prose.
//!
//! Each module mirrors the SHAPE of one emission kind the block documents
//! (task, channel) so `cargo build --target thumbv8m...` type-checks the worked
//! examples in `../codegen/rust/README.md` against the pinned embassy API.
//! CI-only; never inlined. A no_std LIB: it type-checks every example without
//! linking an executor or a runner.
#![no_std]

pub mod channel;
pub mod task;
