//! `toggle` — bool parameter, sticky. nih_plug_egui ships no `ParamButton`, so
//! the documented emission is a plain egui checkbox wired through the setter.
use nih_plug::prelude::ParamSetter;
use nih_plug_egui::egui;

use crate::params::RefParams;

pub fn render(ui: &mut egui::Ui, setter: &ParamSetter, params: &RefParams) {
    ui.label("on");
    let mut value = params.on.value();
    if ui.checkbox(&mut value, "on").changed() {
        setter.begin_set_parameter(&params.on);
        setter.set_parameter(&params.on, value);
        setter.end_set_parameter(&params.on);
    }
    ui.end_row();
}
