//! `radio_group` — enum parameter, expanded. Mirrors the README's stanza, fixed
//! to iterate enum *values* via `from_index` (the documented `for variant in
//! variants()` is a type error: `variants()` yields `&str`, not the enum).
use nih_plug::prelude::{Enum, ParamSetter};
use nih_plug_egui::egui;

use crate::params::{FilterKind, RefParams};

pub fn render(ui: &mut egui::Ui, setter: &ParamSetter, params: &RefParams) {
    ui.label("filter");
    let mut current = params.filter.value();
    let names = FilterKind::variants();
    ui.horizontal(|ui| {
        for (i, name) in names.iter().enumerate() {
            let variant = FilterKind::from_index(i);
            if ui.radio_value(&mut current, variant, *name).clicked() {
                setter.begin_set_parameter(&params.filter);
                setter.set_parameter(&params.filter, variant);
                setter.end_set_parameter(&params.filter);
            }
        }
    });
    ui.end_row();
}
