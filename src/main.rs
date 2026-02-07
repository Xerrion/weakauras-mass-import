//! WeakAura Mass Import Tool
//!
//! A GUI application to mass import WeakAura strings into WoW SavedVariables.

mod app;
mod categories;
mod decoder;
mod error;
mod lua_parser;
mod saved_variables;
mod theme;

use app::WeakAuraImporter;
use eframe::egui;

fn main() -> eframe::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("WeakAura Mass Import"),
        ..Default::default()
    };

    eframe::run_native(
        "WeakAura Mass Import",
        options,
        Box::new(|cc| Ok(Box::new(WeakAuraImporter::new(cc)))),
    )
}
