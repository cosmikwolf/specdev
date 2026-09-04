//! `series` — one `[[egui_plot.series]]` entry becomes one plot item
//! added inside its plot's build closure.
//!
//! Mirrors the README worked example. Where the samples come from is not
//! modelled by this block; every function here takes them as a parameter.

use crate::egui;
use egui_plot::{
    Bar, BarChart, HLine, Line, LineStyle, MarkerShape, Plot, PlotPoint, PlotUi, Points, VLine,
};

/// The `style` enum, one variant per schema value.
pub fn line_style(value: &str) -> LineStyle {
    match value {
        "dashed_dense" => LineStyle::dashed_dense(),
        "dashed_loose" => LineStyle::dashed_loose(),
        "dotted_dense" => LineStyle::dotted_dense(),
        "dotted_loose" => LineStyle::dotted_loose(),
        _ => LineStyle::Solid,
    }
}

/// The `marker` enum, one variant per schema value.
pub fn marker_shape(value: &str) -> MarkerShape {
    match value {
        "diamond" => MarkerShape::Diamond,
        "square" => MarkerShape::Square,
        "cross" => MarkerShape::Cross,
        "plus" => MarkerShape::Plus,
        "up" => MarkerShape::Up,
        "down" => MarkerShape::Down,
        "left" => MarkerShape::Left,
        "right" => MarkerShape::Right,
        "asterisk" => MarkerShape::Asterisk,
        _ => MarkerShape::Circle,
    }
}

/// `color = "#RRGGBB"` resolved at codegen time into a literal.
pub fn rgb(red: u8, green: u8, blue: u8) -> egui::Color32 {
    egui::Color32::from_rgb(red, green, blue)
}

/// `kind = "line"`, owned samples.
///
/// The label is the **first** constructor argument, not a builder call.
/// It is also the item's identity: `Line::new` derives the item id from
/// it, so two series sharing a label share one legend entry and toggle
/// together.
pub fn line(plot_ui: &mut PlotUi<'_>, label: &str, samples: Vec<[f64; 2]>) {
    plot_ui.line(
        Line::new(label.to_owned(), samples)
            .color(rgb(0x4c, 0xaf, 0x50))
            .width(1.5)
            .style(LineStyle::Solid),
    );
}

/// `kind = "line"`, borrowed samples — the zero-copy path for a live
/// viewer that redraws every frame.
///
/// Note the storage type: there is no conversion from `&[[f64; 2]]`, so
/// a buffer meant to be borrowed must be stored as `Vec<PlotPoint>` and
/// passed as a slice. Storing `Vec<[f64; 2]>` forces a clone per frame.
///
/// The lifetime is **not** elidable. `PlotUi<'a>` is invariant behind a
/// mutable reference, so the samples must live exactly as long as the
/// plot's own lifetime parameter, and both must be named.
pub fn line_borrowed<'a>(plot_ui: &mut PlotUi<'a>, label: &str, samples: &'a [PlotPoint]) {
    plot_ui.line(Line::new(label.to_owned(), samples).width(1.5));
}

/// `kind = "points"`.
pub fn points(plot_ui: &mut PlotUi<'_>, label: &str, samples: Vec<[f64; 2]>) {
    plot_ui.points(
        Points::new(label.to_owned(), samples)
            .shape(MarkerShape::Circle)
            .radius(3.0)
            .filled(true)
            .color(rgb(0x21, 0x96, 0xf3)),
    );
}

/// `kind = "h_line"` — a horizontal reference line at a constant y.
pub fn h_line(plot_ui: &mut PlotUi<'_>, label: &str, y: f64) {
    plot_ui.hline(
        HLine::new(label.to_owned(), y)
            .color(rgb(0x9e, 0x9e, 0x9e))
            .width(1.0)
            .style(LineStyle::Solid),
    );
}

/// `kind = "v_line"` — a vertical reference line at a constant x.
pub fn v_line(plot_ui: &mut PlotUi<'_>, label: &str, x: f64) {
    plot_ui.vline(
        VLine::new(label.to_owned(), x)
            .color(rgb(0xff, 0xc1, 0x07))
            .style(LineStyle::dashed_dense()),
    );
}

/// `kind = "bar_chart"`.
///
/// `Bar::new` takes position and height only — no label, unlike every
/// other item constructor. The chart carries the name.
pub fn bar_chart(plot_ui: &mut PlotUi<'_>, label: &str, buckets: &[(f64, f64)]) {
    let bars: Vec<Bar> = buckets
        .iter()
        .map(|&(argument, height)| Bar::new(argument, height))
        .collect();

    plot_ui.bar_chart(BarChart::new(label.to_owned(), bars).color(rgb(0x67, 0x3a, 0xb7)));
}

/// A whole plot with several series, which is the shape codegen emits:
/// the plot's build closure adds each series that names it.
pub fn draw_plot_with_series(ui: &mut egui::Ui, signals: &[(String, Vec<[f64; 2]>)]) {
    Plot::new("signals_multi")
        .legend(egui_plot::Legend::default())
        .show(ui, |plot_ui: &mut PlotUi<'_>| {
            for (label, samples) in signals {
                plot_ui.line(Line::new(label.clone(), samples.clone()).width(1.5));
            }

            plot_ui.hline(HLine::new("zero", 0.0).width(1.0));
        });
}
