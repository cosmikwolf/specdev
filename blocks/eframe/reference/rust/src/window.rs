//! `window` — the `[eframe]` scalars becoming NativeOptions plus the root
//! ViewportBuilder. Window geometry lives on the viewport builder; the
//! rest are NativeOptions fields.
use eframe::egui::{vec2, ViewportBuilder};
use eframe::{NativeOptions, Renderer};

pub fn options() -> NativeOptions {
    NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("iroh p2p test")
            .with_app_id("dev.zetaohm.p2ptest")
            .with_inner_size(vec2(900.0, 600.0))
            .with_min_inner_size(vec2(600.0, 400.0))
            .with_resizable(true),
        centered: true,
        persist_window: true,
        renderer: Renderer::Glow,
        ..Default::default()
    }
}
