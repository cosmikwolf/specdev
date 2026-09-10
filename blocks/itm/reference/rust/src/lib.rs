//! Compiled reference examples for the itm block codegen prose.
//!
//! Type-checks the worked examples in `../codegen/rust/README.md` against
//! the pinned itm API. A std host-target lib: the decoder reads from any
//! `std::io::Read`, so every example type-checks with a byte slice
//! standing in for a live stream. CI-only; never inlined.
//!
//! Each module mirrors one part of the `[itm]` table:
//!   `decoder`        — the `[itm.decoder]` singleton: iterator mode,
//!                      timestamp clock, EOF and malformed-packet policy
//!   `stimulus_ports` — payload decoding for software instrumentation
//!   `data_watches`   — payload decoding for DWT comparators
// A dependency declared but never imported is one the unit contributes to
// user projects with no compiled witness that it is needed or that its
// version is right. `deny` rather than `warn` so the build fails wherever
// it runs -- the reference-crate CI script uses a plain `cargo build`, and
// a warning nobody reads catches nothing.
#![deny(unused_crate_dependencies)]

pub mod data_watches;
pub mod decoder;
pub mod stimulus_ports;
