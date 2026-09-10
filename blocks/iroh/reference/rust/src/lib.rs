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
// right. `deny` rather than `warn` so the build fails wherever it runs -- the
// reference-crate CI script uses a plain `cargo build`, and a warning nobody
// reads catches nothing. If a dep is genuinely used only under a cfg, add
// `use <crate> as _;` here rather than downgrading this.
#![deny(unused_crate_dependencies)]

pub mod channel;
pub mod endpoint;
pub mod protocol;
pub mod stream_bidi;
pub mod stream_uni;
pub mod ticket;
