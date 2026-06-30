//! `knob` — float/int parameter, rotary. The pinned stack has no rotary knob
//! primitive (egui_extras ships none), so the documented emission renders a
//! `ParamSlider` styled as a knob fallback. Mirrors the README's knob stanza.
use nih_plug::prelude::ParamSetter;
use nih_plug_egui::egui;
use nih_plug_egui::widgets::ParamSlider;

use crate::params::RefParams;

pub fn render(ui: &mut egui::Ui, setter: &ParamSetter, params: &RefParams) {
    ui.label("gain");
    // No rotary primitive in the pinned stack: emit a ParamSlider fallback.
    ui.add(ParamSlider::for_param(&params.gain, setter));
    ui.end_row();
}
