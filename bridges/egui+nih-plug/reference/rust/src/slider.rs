//! `slider` — float/int parameter, linear. Mirrors the README's
//! `widgets::ParamSlider::for_param(&self.params.<id>, setter)` stanza.
use nih_plug::prelude::ParamSetter;
use nih_plug_egui::egui;
use nih_plug_egui::widgets::ParamSlider;

use crate::params::RefParams;

pub fn render(ui: &mut egui::Ui, setter: &ParamSetter, params: &RefParams) {
    ui.label("gain");
    ui.add(ParamSlider::for_param(&params.gain, setter));
    ui.end_row();
}
