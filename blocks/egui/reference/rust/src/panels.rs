//! `panels` — the five `[[egui.panels]]` kinds against the MODERN egui panel
//! API (0.35+): unified `egui::Panel`, `show` taking a `&mut Ui`, sizes via
//! `default_size`. This is the form the `eframe` host supplies.
//!
//! The emitted function takes the `&mut Ui` the host hands its paint
//! callback (eframe's `App::ui` does exactly this). Side/top/bottom panels
//! emit before central.
use crate::egui;

pub fn draw(ui: &mut egui::Ui) {
    egui::Panel::top("header").default_size(40.0).show(ui, |_ui| {});
    egui::Panel::bottom("status").default_size(20.0).show(ui, |_ui| {});
    egui::Panel::left("nav").default_size(120.0).show(ui, |_ui| {});
    egui::Panel::right("inspector").default_size(120.0).show(ui, |_ui| {});
    egui::CentralPanel::default().show(ui, |_ui| {});
}
