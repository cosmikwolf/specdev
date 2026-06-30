//! `panels` — the five `[[egui.panels]]` kinds, each mapping to its egui panel
//! type (the README panel table). Side/top panels emit before central.
use nih_plug_egui::egui;

pub fn show(ctx: &egui::Context) {
    egui::TopBottomPanel::top("header").default_height(40.0).show(ctx, |_ui| {});
    egui::TopBottomPanel::bottom("status").default_height(20.0).show(ctx, |_ui| {});
    egui::SidePanel::left("nav").default_width(120.0).show(ctx, |_ui| {});
    egui::SidePanel::right("inspector").default_width(120.0).show(ctx, |_ui| {});
    egui::CentralPanel::default().show(ctx, |_ui| {});
}
