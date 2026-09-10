//! Compiled reference examples for the egui block codegen prose.
//!
//! Type-checks the panel + region emission worked examples in
//! `../codegen/rust/README.md`. A std host-target lib: it type-checks every
//! call without linking a rendering backend. CI-only; never inlined.
//!
//! The block is host-neutral and contributes no crates, so this crate stands
//! in for a host: it declares egui directly and re-exports it below, which is
//! the same crate-root line every real host emits (`pub use eframe::egui;` /
//! `pub use nih_plug_egui::egui;`). Every module below then reaches egui
//! through `crate::egui`, exactly as emitted code does.
//!
//! It stands in for BOTH hosts, because they straddle egui's panel-API break
//! and the prose documents both forms. `panels` compiles the modern form
//! (0.35+, unified `Panel`, `show` taking a `Ui`); `panels_legacy` compiles
//! the pre-0.34 form (`SidePanel`/`TopBottomPanel`, `show` taking a
//! `Context`). Region scaffolding is shared: `regions` compiles it once
//! against the modern egui, and the bridge's own reference compiles the same
//! `Ui` calls against the legacy one.
// A dependency declared but never imported is one the unit contributes to user
// projects with no compiled witness that it is needed or that its version is
// right. `deny` rather than `warn` so the build fails wherever it runs -- the
// reference-crate CI script uses a plain `cargo build`, and a warning nobody
// reads catches nothing. If a dep is genuinely used only under a cfg, add
// `use <crate> as _;` here rather than downgrading this.
#![deny(unused_crate_dependencies)]

/// The host's obligation, standing in for `pub use eframe::egui;`.
/// Emitted code refers to `crate::egui`.
pub use egui;

/// The same obligation for a host on a pre-0.34 egui, standing in for
/// `pub use nih_plug_egui::egui;`. Emitted code still refers to
/// `crate::egui` -- only one host is ever inlined, so only one of these two
/// re-exports exists in a real project. Both are here so both prose forms
/// have a compiled witness.
pub use egui_legacy;

pub mod panels;
pub mod panels_legacy;
pub mod regions;
