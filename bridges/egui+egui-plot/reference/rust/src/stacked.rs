//! `stacked` — several plots in one region, at declared heights and on
//! a shared axis.
//!
//! Mirrors the README worked example. This is the case that makes
//! `fill` and `order` load-bearing rather than decorative.

use crate::egui;
use egui_plot::{Legend, Plot, PlotResponse, PlotUi};

/// One bound plot, resolved at codegen time from the binding.
pub struct Bound {
    pub id: &'static str,
    pub height: f32,
    pub link_group: Option<&'static str>,
}

/// Several plots stacked in one region, in `order`.
///
/// Every plot must carry an explicit height. With `fill = "available"`
/// the first plot would consume the region and the rest would render
/// into nothing — visible as a blank area rather than as an error, which
/// is why the codegen rule exists.
///
/// A shared `link_group` keeps the time axes in step, which is the
/// reason to stack plots rather than merge their series: independent
/// vertical scales, one shared x.
pub fn draw_stack(ui: &mut egui::Ui, plots: &[Bound], now: f64, window: f64) {
    ui.vertical(|ui| {
        for bound in plots {
            let mut plot = Plot::new(bound.id)
                .height(bound.height)
                .legend(Legend::default())
                .allow_drag(egui::Vec2b::new(true, false))
                .allow_zoom(egui::Vec2b::new(true, false))
                .auto_bounds(egui::Vec2b::new(false, true));

            if let Some(group) = bound.link_group {
                plot = plot
                    .link_axis(group, egui::Vec2b::new(true, false))
                    .link_cursor(group, egui::Vec2b::new(true, false));
            }

            let _response: PlotResponse<()> = plot.show(ui, |plot_ui: &mut PlotUi<'_>| {
                plot_ui.set_plot_bounds_x((now - window)..=now);
            });
        }
    });
}

/// The same stack inside a scroll area, for more plots than fit.
///
/// A scroll area gives its content unbounded height, so a plot with no
/// explicit height inside one expands without limit. The declared
/// heights are what keep it finite.
pub fn draw_scrollable_stack(ui: &mut egui::Ui, plots: &[Bound]) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        for bound in plots {
            let _response: PlotResponse<()> = Plot::new(bound.id)
                .height(bound.height)
                .show(ui, |_plot_ui: &mut PlotUi<'_>| {});
        }
    });
}
