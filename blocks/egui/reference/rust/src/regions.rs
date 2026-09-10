//! `regions` — the per-`purpose` region scaffolds and the three layout
//! wrappers from the README's "Per-region scaffolding by purpose" and
//! "Layout direction emission" sections.
//!
//! Every body here is the shape the block emits, minus the hand-authored
//! content: what is being compiled is the egui call surface the scaffolds
//! rely on (`Grid` / `ComboBox` / `button` / `label` / the layout closures),
//! so a scaffold naming a method egui does not have fails this build.
use std::sync::atomic::{AtomicU32, Ordering};

use crate::egui;

/// `layout = "grid"` wrapping a `parameter_controls` region. The BRIDGE FILL
/// marker is where a bridge composes its widget stanzas; `end_row` is the
/// per-stanza terminator those stanzas emit.
pub fn parameter_controls_grid(ui: &mut egui::Ui) {
    egui::Grid::new("param_grid").num_columns(2).show(ui, |ui| {
        // BRIDGE FILL: value-bound widgets for region "param_grid" emit here.
        ui.end_row();
    });
}

/// `layout = "vertical"` — the default when `layout` is omitted.
pub fn parameter_controls_vertical(ui: &mut egui::Ui) {
    ui.vertical(|_ui| {
        // BRIDGE FILL: value-bound widgets for region "<id>" emit here.
    });
}

/// `layout = "horizontal"`.
pub fn parameter_controls_horizontal(ui: &mut egui::Ui) {
    ui.horizontal(|_ui| {
        // BRIDGE FILL: value-bound widgets for region "<id>" emit here.
    });
}

/// `purpose = "metering"` — reads a cell published by a producing thread.
/// The cell reaches the emitted function as a parameter.
pub fn metering(ui: &mut egui::Ui, level: &AtomicU32) {
    ui.vertical(|ui| {
        let level = f32::from_bits(level.load(Ordering::Relaxed));
        // TODO: render -- bar / LED / numeric per author preference.
        let _ = level;
        ui.label("METERING (hand-authored)");
    });
}

/// `purpose = "preset_selector"` — combo box + prev/next + save.
pub fn preset_selector(ui: &mut egui::Ui, current: &str) {
    ui.horizontal(|ui| {
        if ui.button("◀").clicked() {
            // TODO: previous entry
        }
        egui::ComboBox::from_label("Preset")
            .selected_text(current)
            .show_ui(ui, |_ui| {
                // TODO: enumerate entries
            });
        if ui.button("▶").clicked() {
            // TODO: next entry
        }
        if ui.button("Save").clicked() {
            // TODO: save current
        }
    });
}

/// `purpose = "free_form"` — marker plus an open body.
pub fn free_form(ui: &mut egui::Ui) {
    ui.vertical(|_ui| {
        // FREE-FORM REGION: <region.id>
        // Hand-authored body.
    });
}

/// `purpose = "label_block"` — static text.
pub fn label_block(ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        // LABEL BLOCK: <region.id>
        ui.label("Section Header");
        ui.label("Sub-text or instructions");
    });
}
