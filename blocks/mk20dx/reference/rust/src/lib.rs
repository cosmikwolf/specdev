//! Compiled reference examples for the mk20dx block codegen prose.
//!
//! Each module mirrors the SHAPE of one emission kind the block documents
//! (pins, port interrupts, other vectors) so `cargo build` — cross-compiled
//! per `.cargo/config.toml` — type-checks the worked examples in
//! `../codegen/rust/README.md` against the published mk20dx-hal API.
//! CI-only; never inlined. A no_std LIB: it type-checks every example
//! without linking an entry point, so no panic handler or memory map is
//! needed.
// A dependency declared but never imported is one the unit contributes to
// user projects with no compiled witness that it is needed or that its
// version is right. `deny` rather than `warn` so the build fails wherever
// it runs — the reference-crate CI script uses a plain `cargo build`, and
// a warning nobody reads catches nothing.
#![deny(unused_crate_dependencies)]
#![no_std]

pub mod pins;
pub mod port_interrupt;
pub mod vector_interrupt;
