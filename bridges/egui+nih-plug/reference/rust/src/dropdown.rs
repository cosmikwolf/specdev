//! `dropdown` — enum parameter, collapsed. nih_plug_egui ships no
//! `ParamComboBox`, so the documented emission is a hand-rolled `egui::ComboBox`
//! driven by the parameter's `#[derive(Enum)]` (index-based, since
//! `Enum::variants()` yields names and `from_index` reconstructs the value).
use nih_plug::prelude::{Enum, ParamSetter};
use nih_plug_egui::egui;

use crate::params::{FilterKind, RefParams};

pub fn render(ui: &mut egui::Ui, setter: &ParamSetter, params: &RefParams) {
    ui.label("filter");
    let mut current = params.filter.value();
    let names = FilterKind::variants();
    egui::ComboBox::from_label("filter")
        .selected_text(names[current.to_index()])
        .show_ui(ui, |ui| {
            for (i, name) in names.iter().enumerate() {
                let variant = FilterKind::from_index(i);
                if ui.selectable_value(&mut current, variant, *name).clicked() {
                    setter.begin_set_parameter(&params.filter);
                    setter.set_parameter(&params.filter, current);
                    setter.end_set_parameter(&params.filter);
                }
            }
        });
    ui.end_row();
}
