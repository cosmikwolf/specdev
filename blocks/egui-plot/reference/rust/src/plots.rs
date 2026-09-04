//! `plots` — one `[[egui_plot.plots]]` entry becomes one
//! `egui_plot::Plot` with its axes, legend, interaction masks and bounds
//! policy.
//!
//! Mirrors the README worked example. Types are annotated explicitly so
//! the compiler checks them.

use crate::egui;
use egui_plot::{Corner, Legend, Plot, PlotResponse, PlotUi};

/// The `legend` enum, one variant per schema value. `none` is absence of
/// a legend, so it is modelled here as `Option`.
pub fn legend_corner(value: &str) -> Option<Corner> {
    match value {
        "left_top" => Some(Corner::LeftTop),
        "right_top" => Some(Corner::RightTop),
        "left_bottom" => Some(Corner::LeftBottom),
        "right_bottom" => Some(Corner::RightBottom),
        _ => None,
    }
}

/// The `Axes` enum, one `Vec2b` per schema value. Every interaction and
/// bounds setting is a per-axis mask, which is how an axis is "locked":
/// there is no lock method, only a mask that omits it.
pub fn axis_mask(value: &str) -> egui::Vec2b {
    match value {
        "x" => egui::Vec2b::new(true, false),
        "y" => egui::Vec2b::new(false, true),
        "both" => egui::Vec2b::new(true, true),
        _ => egui::Vec2b::new(false, false),
    }
}

/// A plot with every declarative field set, and a `follow_window`.
///
/// `Plot::new` takes an **id source, not a title**: it is never drawn.
/// egui_plot persists pan and zoom state under it, so a plot renamed
/// between releases loses the view the user had set up.
pub fn draw_following(ui: &mut egui::Ui, now: f64, window: f64) {
    let legend = Legend::default().position(Corner::RightTop);

    let _response: PlotResponse<()> = Plot::new("signals")
        .legend(legend)
        .x_axis_label("t [s]")
        .y_axis_label("value")
        .height(240.0)
        .allow_drag(egui::Vec2b::new(true, false))
        .allow_zoom(egui::Vec2b::new(true, false))
        .allow_scroll(egui::Vec2b::new(true, false))
        .auto_bounds(egui::Vec2b::new(false, true))
        .link_axis("scope_time", egui::Vec2b::new(true, false))
        .link_cursor("scope_time", egui::Vec2b::new(true, false))
        .show(ui, |plot_ui: &mut PlotUi<'_>| {
            // `follow_window`: pin the x bounds to the most recent window
            // on every frame. This necessarily takes x off auto-bounds,
            // which is why `auto_bounds` above masks x out.
            plot_ui.set_plot_bounds_x((now - window)..=now);
        });
}

/// A plot with no legend and no follow window — the static case, where
/// the user drives both axes.
pub fn draw_static(ui: &mut egui::Ui) {
    let _response: PlotResponse<()> = Plot::new("histogram")
        .x_axis_label("bucket")
        .allow_drag(egui::Vec2b::new(true, true))
        .allow_zoom(egui::Vec2b::new(true, true))
        .auto_bounds(egui::Vec2b::new(true, true))
        .show(ui, |_plot_ui: &mut PlotUi<'_>| {});
}

/// Reading back what the user is looking at, from the plot's response.
///
/// `hovered_plot_item` carries the `egui::Id` of the item under the
/// pointer, which is derived from the item's constructor name.
pub fn draw_with_hover(ui: &mut egui::Ui) -> Option<egui::Id> {
    let response: PlotResponse<()> = Plot::new("signals_hover")
        .legend(Legend::default())
        .show(ui, |_plot_ui: &mut PlotUi<'_>| {});

    response.hovered_plot_item
}
