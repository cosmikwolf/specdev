//! `host_alias` — this block's host obligation for the `egui` block, and the
//! handoff between them (README "Host obligation — the crate-root egui
//! alias").
//!
//! Three things compile here that nothing else witnesses:
//!   1. `pub use eframe::egui;` at the crate root resolves (see `lib.rs`),
//!      so emitted topology code can reach egui as `crate::egui`.
//!   2. The egui block's MODERN panel form type-checks against the egui this
//!      host actually supplies (0.36 via eframe) -- unified `Panel`, `show`
//!      taking a `&mut Ui`, `default_size`.
//!   3. `App::ui` hands out exactly the `&mut Ui` that form needs, so the
//!      handoff is a direct call with no adapter.
use eframe::{App, Frame};

use crate::egui;

/// Stands in for what the `egui` block emits for a module carrying
/// `[egui]`, modern form.
pub fn draw(ui: &mut egui::Ui) {
    egui::Panel::top("header").default_size(40.0).show(ui, |_ui| {});
    egui::CentralPanel::default().show(ui, |_ui| {});
}

pub struct HostedApp;

impl App for HostedApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        draw(ui);
    }
}
