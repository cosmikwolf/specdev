//! `region_plots` — one plot drawn in one region's body.
//!
//! Mirrors the README worked example. The seam is the `&mut Ui` that a
//! region body already has and that `Plot::show` already wants: the two
//! blocks meet there and need no adapter.

use crate::egui;
use egui_plot::{Corner, Legend, Plot, PlotResponse, PlotUi};

/// The `fill` binding field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fill {
    Available,
    PlotHeight,
}

/// `fill = "available"` — the plot takes the region's full space.
///
/// The region scaffold the egui block emits for a `free_form` region
/// hands its body a `&mut Ui`. That is exactly what `Plot::show` takes,
/// so the binding emits the plot builder directly into that body with
/// nothing in between.
pub fn draw_region(ui: &mut egui::Ui) {
    // Region scaffold (from the interface topology), body filled by this
    // binding rather than hand-authored.
    ui.vertical(|ui| {
        let _response: PlotResponse<()> = Plot::new("signals")
            .legend(Legend::default().position(Corner::RightTop))
            .x_axis_label("t [s]")
            .y_axis_label("value")
            .allow_drag(egui::Vec2b::new(true, false))
            .allow_zoom(egui::Vec2b::new(true, false))
            .auto_bounds(egui::Vec2b::new(false, true))
            .show(ui, |_plot_ui: &mut PlotUi<'_>| {});
    });
}

/// `fill = "plot_height"` — the plot keeps its own declared height.
///
/// Emit `.height(...)` from the plot's own `height` field. Without it a
/// plot takes everything available, which is right for one plot in a
/// region and wrong the moment there are two.
pub fn draw_region_fixed_height(ui: &mut egui::Ui, height: f32) {
    ui.vertical(|ui| {
        let _response: PlotResponse<()> = Plot::new("signals_fixed")
            .height(height)
            .legend(Legend::default())
            .show(ui, |_plot_ui: &mut PlotUi<'_>| {});
    });
}

/// A region whose layout is `horizontal`, from the interface topology.
///
/// The binding does not choose the wrapper — the region's own layout
/// does. The binding only decides what goes inside it.
pub fn draw_region_horizontal(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let _response: PlotResponse<()> = Plot::new("signals_horizontal")
            .height(180.0)
            .show(ui, |_plot_ui: &mut PlotUi<'_>| {});
    });
}
