//! `panels_legacy` — the same five `[[egui.panels]]` kinds against the
//! PRE-0.34 egui panel API: separate `SidePanel` / `TopBottomPanel` /
//! `CentralPanel`, `show` taking a `&Context`, sizes via `default_width` /
//! `default_height`. This is the form the `nih_plug_egui` host supplies
//! (it pins egui 0.31).
//!
//! The emitted function takes the `&Context` that host hands its paint
//! callback. Side/top/bottom panels emit before central, and here that
//! ordering is load-bearing in a way it is not in the modern form: each
//! panel claims its strip from the context in call order.
use crate::egui_legacy as egui;

pub fn draw(ctx: &egui::Context) {
    egui::TopBottomPanel::top("header").default_height(40.0).show(ctx, |_ui| {});
    egui::TopBottomPanel::bottom("status").default_height(20.0).show(ctx, |_ui| {});
    egui::SidePanel::left("nav").default_width(120.0).show(ctx, |_ui| {});
    egui::SidePanel::right("inspector").default_width(120.0).show(ctx, |_ui| {});
    egui::CentralPanel::default().show(ctx, |_ui| {});
}
