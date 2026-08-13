//! `editor` — the host binding this bridge owns: the `create_egui_editor`
//! closure shape emitted into a plugin shell module (README "Host binding").
//! A standalone fn (a lib has no `impl Plugin`); the closure signatures are
//! what matter.
//!
//! It also shows the two host obligations in place: the crate-root alias
//! (see `lib.rs`) and the LEGACY panel form, which is what `nih_plug_egui`'s
//! egui 0.31 requires — `CentralPanel::show` here takes a `&Context`, not a
//! `&mut Ui`. The egui block's prose is the source for that table; this
//! compiles the choice.
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, EguiState};
use std::sync::Arc;

use crate::egui;

#[derive(Default)]
struct EditorState {}

#[derive(Params, Default)]
pub struct ShellParams {}

pub fn build(params: Arc<ShellParams>) -> Option<Box<dyn Editor>> {
    let editor_state = EguiState::from_size(400, 300);
    let _ = params;
    create_egui_editor(
        editor_state,
        EditorState::default(),
        |_ctx, _state| {},
        move |ctx, _setter, _state| {
            // The egui block's panel emission goes here, legacy form.
            egui::TopBottomPanel::top("header")
                .default_height(40.0)
                .show(ctx, |_ui| {});
            egui::CentralPanel::default().show(ctx, |_ui| {});
        },
    )
}
