//! Theme configuration for WeakAura Mass Import
//!
//! Dark theme with WoW-inspired gold accents for iced.

use iced::widget::{button, container, text_input};
use iced::{Border, Color, Theme};

/// Color palette - WoW-inspired dark theme with gold accents
pub mod colors {
    use iced::Color;

    // Backgrounds
    pub const BG_DARKEST: Color = Color::from_rgb(
        0x0d as f32 / 255.0,
        0x0d as f32 / 255.0,
        0x0d as f32 / 255.0,
    );
    pub const BG_DARK: Color = Color::from_rgb(
        0x11 as f32 / 255.0,
        0x11 as f32 / 255.0,
        0x11 as f32 / 255.0,
    );
    pub const BG_PANEL: Color = Color::from_rgb(
        0x1a as f32 / 255.0,
        0x1a as f32 / 255.0,
        0x1a as f32 / 255.0,
    );
    pub const BG_ELEVATED: Color = Color::from_rgb(
        0x22 as f32 / 255.0,
        0x22 as f32 / 255.0,
        0x22 as f32 / 255.0,
    );
    pub const BG_HOVER: Color = Color::from_rgb(
        0x2a as f32 / 255.0,
        0x2a as f32 / 255.0,
        0x2a as f32 / 255.0,
    );
    pub const BG_SELECTED: Color = Color::from_rgb(
        0x33 as f32 / 255.0,
        0x33 as f32 / 255.0,
        0x33 as f32 / 255.0,
    );

    // Accent - WoW Gold
    pub const GOLD: Color = Color::from_rgb(
        0xf8 as f32 / 255.0,
        0xb7 as f32 / 255.0,
        0x00 as f32 / 255.0,
    );
    pub const GOLD_LIGHT: Color = Color::from_rgb(
        0xff as f32 / 255.0,
        0xcc as f32 / 255.0,
        0x33 as f32 / 255.0,
    );
    pub const GOLD_DARK: Color = Color::from_rgb(
        0x8a as f32 / 255.0,
        0x64 as f32 / 255.0,
        0x00 as f32 / 255.0,
    );

    // Text
    pub const TEXT_PRIMARY: Color = Color::from_rgb(
        0xe8 as f32 / 255.0,
        0xe8 as f32 / 255.0,
        0xe8 as f32 / 255.0,
    );
    pub const TEXT_SECONDARY: Color = Color::from_rgb(
        0xa0 as f32 / 255.0,
        0xa0 as f32 / 255.0,
        0xa0 as f32 / 255.0,
    );
    pub const TEXT_MUTED: Color = Color::from_rgb(
        0x70 as f32 / 255.0,
        0x70 as f32 / 255.0,
        0x70 as f32 / 255.0,
    );

    // Status colors
    pub const SUCCESS: Color = Color::from_rgb(
        0x4c as f32 / 255.0,
        0xaf as f32 / 255.0,
        0x50 as f32 / 255.0,
    );
    pub const ERROR: Color = Color::from_rgb(
        0xff as f32 / 255.0,
        0x44 as f32 / 255.0,
        0x44 as f32 / 255.0,
    );

    // Borders
    pub const BORDER: Color = Color::from_rgb(
        0x3a as f32 / 255.0,
        0x3a as f32 / 255.0,
        0x3a as f32 / 255.0,
    );
}

/// Create the custom iced theme
pub fn create_theme() -> Theme {
    Theme::custom(
        "WeakAura Dark".to_string(),
        iced::theme::Palette {
            background: colors::BG_DARK,
            text: colors::TEXT_PRIMARY,
            primary: colors::GOLD,
            success: colors::SUCCESS,
            warning: colors::GOLD_DARK, // WoW-like warning color
            danger: colors::ERROR,
        },
    )
}

/// Custom button style - primary gold button
pub fn button_primary(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(colors::GOLD.into()),
        text_color: colors::BG_DARKEST,
        border: Border::default().rounded(4.0),
        ..Default::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(colors::GOLD_LIGHT.into()),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(colors::GOLD_DARK.into()),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(colors::BG_ELEVATED.into()),
            text_color: colors::TEXT_MUTED,
            ..base
        },
    }
}

/// Custom button style - secondary button
pub fn button_secondary(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(colors::BG_ELEVATED.into()),
        text_color: colors::TEXT_PRIMARY,
        border: Border::default()
            .rounded(4.0)
            .color(colors::BORDER)
            .width(1.0),
        ..Default::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(colors::BG_HOVER.into()),
            text_color: colors::GOLD_LIGHT,
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(colors::BG_SELECTED.into()),
            ..base
        },
        button::Status::Disabled => button::Style {
            text_color: colors::TEXT_MUTED,
            ..base
        },
    }
}

/// Custom button style - danger/error button
pub fn button_danger(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(colors::ERROR.into()),
        text_color: colors::BG_DARKEST,
        border: Border::default().rounded(4.0),
        ..Default::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Color::from_rgb(1.0, 0.4, 0.4).into()),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Color::from_rgb(0.8, 0.2, 0.2).into()),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(colors::BG_ELEVATED.into()),
            text_color: colors::TEXT_MUTED,
            ..base
        },
    }
}

/// Custom button style - small/frameless button
pub fn button_frameless(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: None,
        text_color: colors::TEXT_SECONDARY,
        border: Border::default(),
        ..Default::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            text_color: colors::GOLD_LIGHT,
            ..base
        },
        button::Status::Pressed => button::Style {
            text_color: colors::GOLD,
            ..base
        },
        button::Status::Disabled => button::Style {
            text_color: colors::TEXT_MUTED,
            ..base
        },
    }
}

/// Custom container style - panel
pub fn container_panel(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(colors::BG_PANEL.into()),
        border: Border::default()
            .rounded(0.0)
            .color(colors::BORDER)
            .width(1.0),
        ..Default::default()
    }
}

/// Custom container style - elevated (like a card)
pub fn container_elevated(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(colors::BG_ELEVATED.into()),
        border: Border::default()
            .rounded(4.0)
            .color(colors::BORDER)
            .width(1.0),
        ..Default::default()
    }
}

/// Custom container style - modal overlay background
pub fn container_modal_backdrop(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.7).into()),
        ..Default::default()
    }
}

/// Custom container style - modal dialog
pub fn container_modal(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(colors::BG_PANEL.into()),
        border: Border::default()
            .rounded(8.0)
            .color(colors::BORDER)
            .width(1.0),
        ..Default::default()
    }
}

/// Custom text input style
pub fn text_input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let base = text_input::Style {
        background: colors::BG_ELEVATED.into(),
        border: Border::default()
            .rounded(4.0)
            .color(colors::BORDER)
            .width(1.0),
        icon: colors::TEXT_MUTED,
        placeholder: colors::TEXT_MUTED,
        value: colors::TEXT_PRIMARY,
        selection: colors::GOLD_DARK,
    };

    match status {
        text_input::Status::Active => base,
        text_input::Status::Hovered => text_input::Style {
            border: Border::default()
                .rounded(4.0)
                .color(colors::GOLD_DARK)
                .width(1.0),
            ..base
        },
        text_input::Status::Focused { is_hovered: _ } => text_input::Style {
            border: Border::default()
                .rounded(4.0)
                .color(colors::GOLD)
                .width(2.0),
            ..base
        },
        text_input::Status::Disabled => text_input::Style {
            background: colors::BG_DARK.into(),
            value: colors::TEXT_MUTED,
            ..base
        },
    }
}
