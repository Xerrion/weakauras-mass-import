//! Theme configuration for WeakAura Mass Import
//!
//! Dark theme with WoW-inspired gold accents.

use eframe::egui::{self, Rounding, Stroke, Style, Visuals};

/// Color palette - WoW-inspired dark theme with gold accents
pub mod colors {
    use eframe::egui::Color32;

    // Backgrounds
    pub const BG_DARKEST: Color32 = Color32::from_rgb(0x0d, 0x0d, 0x0d); // #0d0d0d
    pub const BG_DARK: Color32 = Color32::from_rgb(0x11, 0x11, 0x11); // #111111
    pub const BG_PANEL: Color32 = Color32::from_rgb(0x1a, 0x1a, 0x1a); // #1a1a1a
    pub const BG_ELEVATED: Color32 = Color32::from_rgb(0x22, 0x22, 0x22); // #222222
    pub const BG_HOVER: Color32 = Color32::from_rgb(0x2a, 0x2a, 0x2a); // #2a2a2a
    pub const BG_SELECTED: Color32 = Color32::from_rgb(0x33, 0x33, 0x33); // #333333

    // Accent - WoW Gold
    pub const GOLD: Color32 = Color32::from_rgb(0xf8, 0xb7, 0x00); // #f8b700
    pub const GOLD_LIGHT: Color32 = Color32::from_rgb(0xff, 0xcc, 0x33); // #ffcc33
    pub const GOLD_DARK: Color32 = Color32::from_rgb(0x8a, 0x64, 0x00); // #8a6400

    // Text
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xe8, 0xe8, 0xe8); // #e8e8e8
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0xa0, 0xa0, 0xa0); // #a0a0a0
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x70, 0x70, 0x70); // #707070

    // Status colors
    pub const SUCCESS: Color32 = Color32::from_rgb(0x4c, 0xaf, 0x50); // #4caf50
    pub const ERROR: Color32 = Color32::from_rgb(0xff, 0x44, 0x44); // #ff4444

    // Borders
    pub const BORDER: Color32 = Color32::from_rgb(0x3a, 0x3a, 0x3a); // #3a3a3a
}

/// Configure the egui context with our custom theme
pub fn configure_theme(ctx: &egui::Context) {
    let mut style = Style::default();
    let mut visuals = Visuals::dark();

    // Window styling
    visuals.window_fill = colors::BG_PANEL;
    visuals.window_stroke = Stroke::new(1.0, colors::BORDER);
    visuals.window_rounding = Rounding::same(8.0);

    // Panel styling
    visuals.panel_fill = colors::BG_DARK;

    // Widget colors
    visuals.widgets.noninteractive.bg_fill = colors::BG_ELEVATED;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors::TEXT_SECONDARY);
    visuals.widgets.noninteractive.rounding = Rounding::same(4.0);

    visuals.widgets.inactive.bg_fill = colors::BG_ELEVATED;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, colors::TEXT_PRIMARY);
    visuals.widgets.inactive.rounding = Rounding::same(4.0);

    visuals.widgets.hovered.bg_fill = colors::BG_HOVER;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, colors::GOLD_LIGHT);
    visuals.widgets.hovered.rounding = Rounding::same(4.0);

    visuals.widgets.active.bg_fill = colors::GOLD_DARK;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, colors::TEXT_PRIMARY);
    visuals.widgets.active.rounding = Rounding::same(4.0);

    visuals.widgets.open.bg_fill = colors::BG_SELECTED;
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, colors::GOLD);
    visuals.widgets.open.rounding = Rounding::same(4.0);

    // Selection
    visuals.selection.bg_fill = colors::GOLD_DARK;
    visuals.selection.stroke = Stroke::new(1.0, colors::GOLD);

    // Hyperlinks
    visuals.hyperlink_color = colors::GOLD_LIGHT;

    // Extreme background (behind everything)
    visuals.extreme_bg_color = colors::BG_DARKEST;

    // Faint color for subtle backgrounds
    visuals.faint_bg_color = colors::BG_PANEL;

    // Text cursor
    visuals.text_cursor.stroke = Stroke::new(2.0, colors::GOLD);

    // Striped background
    visuals.striped = true;

    // Apply visuals
    style.visuals = visuals;

    // Spacing adjustments
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(12.0);

    ctx.set_style(style);
}

/// Helper to create a step header (e.g., "Step 1: Select File")
pub fn step_header(step: u32, text: &str) -> egui::RichText {
    egui::RichText::new(format!("Step {}: {}", step, text))
        .color(colors::GOLD)
        .size(16.0)
        .strong()
}

/// Helper to create a secondary/muted label
pub fn muted_text(text: &str) -> egui::RichText {
    egui::RichText::new(text).color(colors::TEXT_MUTED)
}
