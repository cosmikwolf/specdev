//! `viewport` — `[[eframe.viewports]]` entries becoming additional OS
//! windows. Note the 0.36 callback signature takes `&mut Ui`, not a
//! Context: the child paints into a Ui the same way the root does.
use eframe::egui::{self, ViewportBuilder, ViewportClass, ViewportId};

/// kind = "deferred" — the child paints on its own schedule, and the
/// callback runs only when that viewport needs repainting. Prefer this:
/// an immediate viewport re-runs inside the parent's pass, so a slow
/// child stalls the root window.
pub fn show_deferred(ctx: &egui::Context) {
    ctx.show_viewport_deferred(
        ViewportId::from_hash_of("event_log"),
        ViewportBuilder::default()
            .with_title("Event log")
            .with_inner_size(egui::vec2(480.0, 320.0)),
        |ui: &mut egui::Ui, _class: ViewportClass| {
            ui.label("child viewport contents");
        },
    );
}
