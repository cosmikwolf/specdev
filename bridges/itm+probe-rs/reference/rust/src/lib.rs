//! Compiled reference examples for the itm+probe-rs bridge codegen prose.
//! Each module mirrors the SHAPE `/spec-gen` emits for one case. CI-only;
//! never inlined.
//!
//!   `stream`        — joining the probe's SWO reader to the decoder, and
//!                     reconciling the timestamp clock
//!   `armed_watches` — arming a declared comparator and routing its
//!                     packets back to the watch that declared it
//!
//! The endpoint types are annotated explicitly throughout, because that
//! is what makes the compiler check them — in particular that
//! `SwoReader<'_>` really does satisfy the decoder's `R: Read` bound,
//! which is the claim the whole pairing rests on.
// A dependency declared but never imported is one the unit contributes to
// user projects with no compiled witness that it is needed or that its
// version is right. `deny` rather than `warn` so the build fails wherever
// it runs -- the reference-crate CI script uses a plain `cargo build`, and
// a warning nobody reads catches nothing.
#![deny(unused_crate_dependencies)]

pub mod armed_watches;
pub mod stream;
