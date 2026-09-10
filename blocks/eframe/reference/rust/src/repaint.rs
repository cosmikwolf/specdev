//! `repaint` — the three `repaint` policies, plus the cross-thread wake.
//!
//! eframe is reactive by default: it paints on input and stops otherwise.
//! Anything arriving from outside the UI thread is therefore invisible
//! until something requests a repaint.
use eframe::egui;
use std::time::Duration;

/// repaint = "reactive" — the default. Emit nothing; eframe paints on
/// input and when someone calls request_repaint.
pub fn reactive(_ctx: &egui::Context) {}

/// repaint = "continuous" — paint every frame, unconditionally. Burns CPU
/// and GPU whether or not anything changed.
pub fn continuous(ctx: &egui::Context) {
    ctx.request_repaint();
}

/// repaint = "interval" — wake again after a fixed delay. Bounds staleness
/// without spinning: useful for a clock or a periodically-polled value.
pub fn interval(ctx: &egui::Context, interval_ms: u64) {
    ctx.request_repaint_after(Duration::from_millis(interval_ms));
}

/// The cross-thread wake. `egui::Context` is cheap to clone and is Send +
/// Sync, so a worker elsewhere holds a clone and calls request_repaint
/// when it has produced something. This is what lets a reactive app show
/// data that arrives without user input, with no polling at all.
pub fn wake_on_external_work(ctx: egui::Context) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        // ... a receiver, a socket, a timer — whatever produced the work.
        ctx.request_repaint();
    })
}
