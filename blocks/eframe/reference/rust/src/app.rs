//! `app` — the App impl and the run_native entry point.
//!
//! eframe 0.36 splits the frame into two calls: `logic` runs once before
//! each `ui` AND whenever a repaint was requested while the window is
//! hidden, and `ui` paints. `ui` is the only REQUIRED method. There is no
//! `update` method — that was the pre-0.35 shape and does not exist here.
use eframe::egui;
use eframe::{App, CreationContext, Frame, NativeOptions};

pub struct DemoApp {
    pub frames: u64,
}

impl App for DemoApp {
    /// Non-painting work. This is where state arriving from elsewhere is
    /// taken up, because it also runs when the window is hidden but a
    /// repaint was requested — so nothing is missed while minimised.
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        let _ = ctx;
        self.frames += 1;
    }

    /// Painting. Called each time the UI needs repainting.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        ui.label(format!("frames: {}", self.frames));
    }
}

pub fn run(options: NativeOptions) -> eframe::Result {
    eframe::run_native(
        "demo-app",
        options,
        Box::new(|cc: &CreationContext<'_>| {
            let _ = cc;
            Ok(Box::new(DemoApp { frames: 0 }))
        }),
    )
}
