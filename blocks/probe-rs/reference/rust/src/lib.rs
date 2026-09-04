//! Compiled reference examples for the probe-rs block codegen prose.
//!
//! Type-checks the worked examples in `../codegen/rust/README.md` against
//! the pinned probe-rs API. A std host-target lib: it type-checks every
//! call without a probe attached. CI-only; never inlined.
//!
//! Each module mirrors one `[probe_rs]` entity kind:
//!   `sessions`      — probe selection and attach
//!   `samplers`      — periodic memory reads against a running core
//!   `trace_sources` — SWO / TPIU / trace-memory setup and draining
//!
//! Fallible functions here return `Box<dyn Error>` so the examples stay
//! about the probe-rs calls rather than about error plumbing. Emitted
//! code maps these into the project's own error type per the project's
//! profile's `module.conventions.md`.
// A dependency declared but never imported is one the unit contributes to
// user projects with no compiled witness that it is needed or that its
// version is right. `deny` rather than `warn` so the build fails wherever
// it runs -- the reference-crate CI script uses a plain `cargo build`, and
// a warning nobody reads catches nothing.
#![deny(unused_crate_dependencies)]

pub mod samplers;
pub mod sessions;
pub mod trace_sources;
