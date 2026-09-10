//! `persistence` — `[[eframe.persisted]]` entries becoming typed storage
//! round-trips. Values are read once in the app constructor from
//! CreationContext::storage and written back in App::save.
//!
//! Requires eframe's `persistence` feature; without it `get_value` and
//! `set_value` do not exist.
use eframe::egui;
use eframe::{App, CreationContext, Frame, Storage};
use serde::{Deserialize, Serialize};

/// One `[[eframe.persisted]]` entry: a key plus the type stored under it.
pub const PREFS_KEY: &str = "prefs";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Prefs {
    pub last_ticket: String,
    pub window_scale: f32,
}

pub struct PersistApp {
    prefs: Prefs,
}

impl PersistApp {
    /// Read at construction. A missing or unparseable value falls back to
    /// Default rather than failing to start — a corrupt preferences file
    /// must never prevent the app from opening.
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let prefs: Prefs = cc
            .storage
            .and_then(|storage| eframe::get_value::<Prefs>(storage, PREFS_KEY))
            .unwrap_or_default();
        Self { prefs }
    }
}

impl App for PersistApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        ui.label(&self.prefs.last_ticket);
    }

    /// Called by eframe on exit and periodically per auto_save_interval.
    fn save(&mut self, storage: &mut dyn Storage) {
        eframe::set_value(storage, PREFS_KEY, &self.prefs);
    }
}
