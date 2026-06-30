//! `button` — bool parameter, momentary. Mirrors the README's
//! `ui.button(...)` + `setter.begin/set/end_set_parameter` stanza (nih-plug has
//! no first-class momentary widget, so this is plain egui + setter calls).
use nih_plug::prelude::ParamSetter;
use nih_plug_egui::egui;

use crate::params::RefParams;

pub fn render(ui: &mut egui::Ui, setter: &ParamSetter, params: &RefParams) {
    ui.label("trigger");
    let response = ui.button("trigger");
    if response.clicked() {
        setter.begin_set_parameter(&params.on);
        setter.set_parameter(&params.on, true);
        setter.end_set_parameter(&params.on);
        // momentary: a real plugin schedules an end-frame reset to false.
    }
    ui.end_row();
}
