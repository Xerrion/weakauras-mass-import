//! WeakAura Mass Import Tool
//!
//! A GUI application to mass import WeakAura strings into WoW SavedVariables.

// Hide console window on Windows release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod categories;
mod decoder;
mod error;
mod lua_parser;
mod saved_variables;
mod theme;
mod util;

use app::WeakAuraImporter;

fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::fmt::init();

    iced::application(
        WeakAuraImporter::new,
        WeakAuraImporter::update,
        WeakAuraImporter::view,
    )
    .title("WeakAuras Mass Importer")
    .theme(WeakAuraImporter::theme)
    .window_size((1000.0, 700.0))
    .run()
}
