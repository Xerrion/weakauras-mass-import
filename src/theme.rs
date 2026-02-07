//! Modern theme configuration for WeakAura Mass Import (2026 Edition)
//!
//! A sophisticated dark theme with WoW-inspired gold accents,
//! featuring modern design language: generous spacing, smooth gradients,
//! rounded corners, and subtle depth effects.

use iced::widget::{button, container, scrollable, text_input};
use iced::{Border, Color, Shadow, Theme, Vector};

/// Color palette - Modern dark theme with WoW gold accents
pub mod colors {
    use iced::Color;

    // Background layers (subtle variation for depth)
    pub const BG_DARKEST: Color = Color::from_rgb(0.035, 0.035, 0.043); // #090909
    pub const BG_DARK: Color = Color::from_rgb(0.055, 0.055, 0.067); // #0e0e11
    pub const BG_PANEL: Color = Color::from_rgb(0.078, 0.082, 0.098); // #141519
    pub const BG_ELEVATED: Color = Color::from_rgb(0.106, 0.110, 0.129); // #1b1c21
    pub const BG_SURFACE: Color = Color::from_rgb(0.133, 0.137, 0.161); // #222329
    pub const BG_HOVER: Color = Color::from_rgb(0.161, 0.165, 0.192); // #292a31
    pub const BG_SELECTED: Color = Color::from_rgb(0.188, 0.192, 0.224); // #303139

    // Accent - WoW Gold (refined gradient stops)
    pub const GOLD: Color = Color::from_rgb(0.973, 0.718, 0.0); // #f8b700
    pub const GOLD_LIGHT: Color = Color::from_rgb(1.0, 0.835, 0.298); // #ffd54c
    pub const GOLD_SOFT: Color = Color::from_rgb(0.973, 0.765, 0.278); // #f8c347
    pub const GOLD_DARK: Color = Color::from_rgb(0.659, 0.478, 0.0); // #a87a00
    pub const GOLD_MUTED: Color = Color::from_rgb(0.376, 0.286, 0.086); // #604916

    // Text hierarchy
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.945, 0.945, 0.957); // #f1f1f4
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.694, 0.702, 0.749); // #b1b3bf
    pub const TEXT_MUTED: Color = Color::from_rgb(0.471, 0.478, 0.533); // #787a88
    pub const TEXT_DISABLED: Color = Color::from_rgb(0.329, 0.337, 0.384); // #545662

    // Semantic colors
    pub const SUCCESS: Color = Color::from_rgb(0.298, 0.808, 0.478); // #4cce7a
    pub const SUCCESS_MUTED: Color = Color::from_rgb(0.176, 0.357, 0.255); // #2d5b41
    pub const ERROR: Color = Color::from_rgb(1.0, 0.376, 0.376); // #ff6060
    pub const ERROR_MUTED: Color = Color::from_rgb(0.357, 0.176, 0.176); // #5b2d2d
    pub const WARNING: Color = Color::from_rgb(1.0, 0.722, 0.298); // #ffb84c
    pub const INFO: Color = Color::from_rgb(0.376, 0.686, 1.0); // #60afff

    // Borders & Dividers
    pub const BORDER: Color = Color::from_rgb(0.196, 0.200, 0.235); // #32333c
    pub const BORDER_LIGHT: Color = Color::from_rgb(0.255, 0.259, 0.298); // #41424c
    pub const BORDER_SUBTLE: Color = Color::from_rgb(0.137, 0.141, 0.169); // #23242b
    pub const DIVIDER: Color = Color::from_rgb(0.118, 0.122, 0.145); // #1e1f25
}

/// Modern design constants
pub mod design {
    /// Corner radius for components
    pub const RADIUS_SMALL: f32 = 6.0;
    pub const RADIUS_MEDIUM: f32 = 10.0;
    pub const RADIUS_LARGE: f32 = 14.0;
    pub const RADIUS_XL: f32 = 20.0;

    /// Shadow definitions
    pub const SHADOW_OFFSET: f32 = 2.0;
    pub const SHADOW_BLUR: f32 = 8.0;
}

/// Create the custom iced theme
pub fn create_theme() -> Theme {
    Theme::custom(
        "WeakAura Dark 2026".to_string(),
        iced::theme::Palette {
            background: colors::BG_DARK,
            text: colors::TEXT_PRIMARY,
            primary: colors::GOLD,
            success: colors::SUCCESS,
            warning: colors::WARNING,
            danger: colors::ERROR,
        },
    )
}

/// Custom button style - primary gold button with modern look
pub fn button_primary(_theme: &Theme, status: button::Status) -> button::Style {
    let shadow = Shadow {
        color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
        offset: Vector::new(0.0, design::SHADOW_OFFSET),
        blur_radius: design::SHADOW_BLUR,
    };

    let base = button::Style {
        background: Some(colors::GOLD.into()),
        text_color: colors::BG_DARKEST,
        border: Border::default().rounded(design::RADIUS_MEDIUM),
        shadow,
        snap: true,
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(colors::GOLD_LIGHT.into()),
            shadow: Shadow {
                color: Color::from_rgba(0.973, 0.718, 0.0, 0.4),
                offset: Vector::new(0.0, design::SHADOW_OFFSET),
                blur_radius: design::SHADOW_BLUR + 4.0,
            },
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(colors::GOLD_DARK.into()),
            shadow: Shadow::default(),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(colors::BG_SURFACE.into()),
            text_color: colors::TEXT_DISABLED,
            shadow: Shadow::default(),
            ..base
        },
    }
}

/// Custom button style - secondary button with subtle appearance
pub fn button_secondary(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(colors::BG_ELEVATED.into()),
        text_color: colors::TEXT_PRIMARY,
        border: Border::default()
            .rounded(design::RADIUS_MEDIUM)
            .color(colors::BORDER)
            .width(1.0),
        shadow: Shadow::default(),
        snap: true,
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(colors::BG_HOVER.into()),
            text_color: colors::GOLD_LIGHT,
            border: Border::default()
                .rounded(design::RADIUS_MEDIUM)
                .color(colors::GOLD_MUTED)
                .width(1.0),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(colors::BG_SELECTED.into()),
            text_color: colors::GOLD,
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(colors::BG_PANEL.into()),
            text_color: colors::TEXT_DISABLED,
            border: Border::default()
                .rounded(design::RADIUS_MEDIUM)
                .color(colors::BORDER_SUBTLE)
                .width(1.0),
            ..base
        },
    }
}

/// Custom button style - danger/error button
pub fn button_danger(_theme: &Theme, status: button::Status) -> button::Style {
    let shadow = Shadow {
        color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
        offset: Vector::new(0.0, design::SHADOW_OFFSET),
        blur_radius: design::SHADOW_BLUR,
    };

    let base = button::Style {
        background: Some(colors::ERROR.into()),
        text_color: colors::BG_DARKEST,
        border: Border::default().rounded(design::RADIUS_MEDIUM),
        shadow,
        snap: true,
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Color::from_rgb(1.0, 0.5, 0.5).into()),
            shadow: Shadow {
                color: Color::from_rgba(1.0, 0.376, 0.376, 0.4),
                offset: Vector::new(0.0, design::SHADOW_OFFSET),
                blur_radius: design::SHADOW_BLUR + 4.0,
            },
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Color::from_rgb(0.8, 0.25, 0.25).into()),
            shadow: Shadow::default(),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(colors::BG_SURFACE.into()),
            text_color: colors::TEXT_DISABLED,
            shadow: Shadow::default(),
            ..base
        },
    }
}

/// Custom button style - frameless/ghost button
pub fn button_frameless(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: None,
        text_color: colors::TEXT_SECONDARY,
        border: Border::default().rounded(design::RADIUS_SMALL),
        shadow: Shadow::default(),
        snap: true,
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
            text_color: colors::GOLD,
            ..base
        },
        button::Status::Disabled => button::Style {
            text_color: colors::TEXT_DISABLED,
            ..base
        },
    }
}

/// Custom button style - icon button (compact, no text styling)
pub fn button_icon(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: None,
        text_color: colors::TEXT_MUTED,
        border: Border::default().rounded(design::RADIUS_SMALL),
        shadow: Shadow::default(),
        snap: true,
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(colors::BG_HOVER.into()),
            text_color: colors::TEXT_PRIMARY,
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(colors::BG_SELECTED.into()),
            text_color: colors::GOLD,
            ..base
        },
        button::Status::Disabled => button::Style {
            text_color: colors::TEXT_DISABLED,
            ..base
        },
    }
}

/// Custom container style - main panel (sidebar/content areas)
pub fn container_panel(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(colors::BG_PANEL.into()),
        border: Border::default().color(colors::BORDER_SUBTLE).width(1.0),
        ..Default::default()
    }
}

/// Custom container style - elevated card with depth
pub fn container_elevated(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(colors::BG_ELEVATED.into()),
        border: Border::default()
            .rounded(design::RADIUS_MEDIUM)
            .color(colors::BORDER)
            .width(1.0),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 6.0,
        },
        text_color: None,
        snap: true,
    }
}

/// Custom container style - surface (slightly elevated content)
pub fn container_surface(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(colors::BG_SURFACE.into()),
        border: Border::default()
            .rounded(design::RADIUS_SMALL)
            .color(colors::BORDER_SUBTLE)
            .width(1.0),
        ..Default::default()
    }
}

/// Custom container style - modal overlay background (frosted glass effect)
pub fn container_modal_backdrop(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgba(0.02, 0.02, 0.03, 0.85).into()),
        ..Default::default()
    }
}

/// Custom container style - modal dialog box
pub fn container_modal(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(colors::BG_PANEL.into()),
        border: Border::default()
            .rounded(design::RADIUS_LARGE)
            .color(colors::BORDER_LIGHT)
            .width(1.0),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        },
        text_color: None,
        snap: true,
    }
}

/// Custom container style - status bar
pub fn container_status_bar(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(colors::BG_DARKEST.into()),
        border: Border::default().color(colors::BORDER_SUBTLE).width(1.0),
        ..Default::default()
    }
}

/// Custom container style - menu/toolbar
pub fn container_toolbar(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(colors::BG_ELEVATED.into()),
        border: Border::default().color(colors::BORDER_SUBTLE).width(1.0),
        ..Default::default()
    }
}

/// Custom container style - success highlight
pub fn container_success(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(colors::SUCCESS_MUTED.into()),
        border: Border::default()
            .rounded(design::RADIUS_SMALL)
            .color(colors::SUCCESS)
            .width(1.0),
        ..Default::default()
    }
}

/// Custom container style - error highlight
pub fn container_error(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(colors::ERROR_MUTED.into()),
        border: Border::default()
            .rounded(design::RADIUS_SMALL)
            .color(colors::ERROR)
            .width(1.0),
        ..Default::default()
    }
}

/// Custom text input style with modern aesthetics
pub fn text_input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let base = text_input::Style {
        background: colors::BG_SURFACE.into(),
        border: Border::default()
            .rounded(design::RADIUS_MEDIUM)
            .color(colors::BORDER)
            .width(1.0),
        icon: colors::TEXT_MUTED,
        placeholder: colors::TEXT_MUTED,
        value: colors::TEXT_PRIMARY,
        selection: colors::GOLD_MUTED,
    };

    match status {
        text_input::Status::Active => base,
        text_input::Status::Hovered => text_input::Style {
            border: Border::default()
                .rounded(design::RADIUS_MEDIUM)
                .color(colors::BORDER_LIGHT)
                .width(1.0),
            ..base
        },
        text_input::Status::Focused { is_hovered: _ } => text_input::Style {
            border: Border::default()
                .rounded(design::RADIUS_MEDIUM)
                .color(colors::GOLD)
                .width(2.0),
            ..base
        },
        text_input::Status::Disabled => text_input::Style {
            background: colors::BG_PANEL.into(),
            value: colors::TEXT_DISABLED,
            ..base
        },
    }
}

/// Custom scrollable style
pub fn scrollable_style(_theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let scroller_active = scrollable::Scroller {
        background: colors::BORDER_LIGHT.into(),
        border: Border::default().rounded(4.0),
    };

    let scroller_hovered = scrollable::Scroller {
        background: colors::GOLD_MUTED.into(),
        border: Border::default().rounded(4.0),
    };

    let rail = scrollable::Rail {
        background: Some(colors::BG_ELEVATED.into()),
        border: Border::default().rounded(4.0),
        scroller: scroller_active,
    };

    let rail_hovered = scrollable::Rail {
        background: Some(colors::BG_SURFACE.into()),
        border: Border::default().rounded(4.0),
        scroller: scroller_hovered,
    };

    let auto_scroll = scrollable::AutoScroll {
        background: colors::BG_SURFACE.into(),
        border: Border::default().rounded(4.0),
        shadow: Shadow::default(),
        icon: colors::TEXT_MUTED,
    };

    match status {
        scrollable::Status::Active { .. } => scrollable::Style {
            container: container::Style::default(),
            vertical_rail: rail,
            horizontal_rail: rail,
            gap: None,
            auto_scroll,
        },
        scrollable::Status::Hovered {
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
            ..
        } => scrollable::Style {
            container: container::Style::default(),
            vertical_rail: if is_vertical_scrollbar_hovered {
                rail_hovered
            } else {
                rail
            },
            horizontal_rail: if is_horizontal_scrollbar_hovered {
                rail_hovered
            } else {
                rail
            },
            gap: None,
            auto_scroll,
        },
        scrollable::Status::Dragged {
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
            ..
        } => {
            let scroller_dragged = scrollable::Scroller {
                background: colors::GOLD.into(),
                border: Border::default().rounded(4.0),
            };
            let rail_dragged = scrollable::Rail {
                background: Some(colors::BG_SURFACE.into()),
                border: Border::default().rounded(4.0),
                scroller: scroller_dragged,
            };
            scrollable::Style {
                container: container::Style::default(),
                vertical_rail: if is_vertical_scrollbar_dragged {
                    rail_dragged
                } else {
                    rail
                },
                horizontal_rail: if is_horizontal_scrollbar_dragged {
                    rail_dragged
                } else {
                    rail
                },
                gap: None,
                auto_scroll,
            }
        }
    }
}

/// Modern typography sizes
pub mod typography {
    pub const TITLE: f32 = 22.0;
    pub const HEADING: f32 = 18.0;
    pub const SUBHEADING: f32 = 15.0;
    pub const BODY: f32 = 14.0;
    pub const CAPTION: f32 = 12.0;
    pub const SMALL: f32 = 11.0;
}

/// Spacing constants for consistent layout
pub mod spacing {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 24.0;
    pub const XXL: f32 = 32.0;
}
