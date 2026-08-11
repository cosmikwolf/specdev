//! Compiled reference examples for the iroh block codegen prose.
//!
//! Each module mirrors the SHAPE of one emission kind the block documents
//! (endpoint, protocol, stream_bidi, stream_uni, channel, ticket) so
//! `cargo build` type-checks the worked examples in
//! `../codegen/rust/README.md` against the pinned iroh 1.0.3 API.
//! CI-only; never inlined. A std host LIB: it type-checks every example
//! without binding a socket or driving a runtime.
// A dependency declared but never imported is one the unit contributes to user
// projects with no compiled witness that it is needed or that its version is
// right. Plugin CI builds reference crates with `-D warnings`.
#![warn(unused_crate_dependencies)]

pub mod channel;
pub mod endpoint;
pub mod protocol;
pub mod stream_bidi;
pub mod stream_uni;
pub mod ticket;
