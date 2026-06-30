//! `editor` — the `create_egui_editor` closure shape the block emits into a
//! plugin shell module (README "Editor closure shape"). A standalone fn (a lib
//! has no `impl Plugin`); the closure signatures are what matter.
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, EguiState};
use std::sync::Arc;

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
            egui::CentralPanel::default().show(ctx, |_ui| {});
        },
    )
}
