//! `value_display` — any parameter, read-only. Mirrors the README's stanza,
//! fixed to use `Param::unit()` (the documented `.unit_string()` does not exist).
use nih_plug::prelude::Param;
use nih_plug_egui::egui;

use crate::params::RefParams;

pub fn render(ui: &mut egui::Ui, params: &RefParams) {
    ui.label("gain");
    ui.label(format!("{}{}", params.gain.value(), params.gain.unit()));
    ui.end_row();
}
