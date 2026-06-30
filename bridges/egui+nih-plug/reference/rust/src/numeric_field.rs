//! `numeric_field` — float/int parameter, typed entry. Mirrors the README's
//! `egui::DragValue::new(&mut value)` + `setter.begin/set/end_set_parameter`
//! stanza.
use nih_plug::prelude::ParamSetter;
use nih_plug_egui::egui;

use crate::params::RefParams;

pub fn render(ui: &mut egui::Ui, setter: &ParamSetter, params: &RefParams) {
    ui.label("gain");
    let mut value = params.gain.value();
    let response = ui.add(egui::DragValue::new(&mut value));
    if response.changed() {
        setter.begin_set_parameter(&params.gain);
        setter.set_parameter(&params.gain, value);
        setter.end_set_parameter(&params.gain);
    }
    ui.end_row();
}
