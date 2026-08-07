//! Compiled reference examples for the iroh block codegen prose.
//!
//! Each module mirrors the SHAPE of one emission kind the block documents
//! (endpoint, protocol, stream_bidi, stream_uni, channel, ticket) so
//! `cargo build` type-checks the worked examples in
//! `../codegen/rust/README.md` against the pinned iroh 1.0.3 API.
//! CI-only; never inlined. A std host LIB: it type-checks every example
//! without binding a socket or driving a runtime.

pub mod channel;
pub mod endpoint;
pub mod protocol;
pub mod stream_bidi;
pub mod stream_uni;
pub mod ticket;
