//! WeakAura Mass Import - 2026 Cyber Dark Theme
//!
//! A bold, immersive dark theme inspired by 2026 design trends:
//! - Cyber gradients with electric neon accents
//! - Neumorphic depth and tactile surfaces  
//! - Bold typography hierarchy
//! - Vibrant "dopamine design" color palette
//! - WoW-inspired gold with modern cyber twist

#![allow(dead_code)]

use iced::widget::{button, container, scrollable, text_input};
use iced::{Border, Color, Shadow, Theme, Vector};

// ============================================================================
// COLOR PALETTE - Cyber Dark with Electric Gold
// ============================================================================

pub mod colors {
    use iced::Color;

    // === BACKGROUND LAYERS (Deep space blacks with subtle blue undertones) ===
    /// Deepest void - for overlays and maximum contrast areas
    pub const BG_VOID: Color = Color::from_rgb(0.020, 0.024, 0.035); // #050609
    /// Base dark - primary application background  
    pub const BG_BASE: Color = Color::from_rgb(0.043, 0.051, 0.071); // #0b0d12
    /// Panel background - sidebars and content areas
    pub const BG_PANEL: Color = Color::from_rgb(0.063, 0.075, 0.102); // #10131a
    /// Elevated surface - cards and raised elements
    pub const BG_ELEVATED: Color = Color::from_rgb(0.086, 0.102, 0.137); // #161a23
    /// Surface - interactive element backgrounds
    pub const BG_SURFACE: Color = Color::from_rgb(0.110, 0.129, 0.173); // #1c212c
    /// Hover state - subtle highlight on interaction
    pub const BG_HOVER: Color = Color::from_rgb(0.137, 0.161, 0.216); // #232937
    /// Selected/active state
    pub const BG_SELECTED: Color = Color::from_rgb(0.165, 0.192, 0.259); // #2a3142

    // === ACCENT - Electric Gold (WoW heritage + cyber glow) ===
    /// Primary gold - main accent, CTAs, active states
    pub const GOLD: Color = Color::from_rgb(1.0, 0.784, 0.0); // #ffc800 - Electric gold
    /// Light gold - hover states, highlights
    pub const GOLD_LIGHT: Color = Color::from_rgb(1.0, 0.878, 0.4); // #ffe066 - Bright glow
    /// Bright gold - maximum emphasis
    pub const GOLD_BRIGHT: Color = Color::from_rgb(1.0, 0.918, 0.6); // #ffea99 - Neon flash
    /// Dark gold - pressed states, shadows
    pub const GOLD_DARK: Color = Color::from_rgb(0.8, 0.588, 0.0); // #cc9600 - Deep amber
    /// Muted gold - subtle accents, selection backgrounds
    pub const GOLD_MUTED: Color = Color::from_rgb(0.4, 0.314, 0.1); // #665019 - Warm glow
    /// Gold glow - for shadow/glow effects
    pub const GOLD_GLOW: Color = Color::from_rgba(1.0, 0.784, 0.0, 0.3); // Translucent gold

    // === SECONDARY ACCENT - Cyber Cyan (complementary) ===
    /// Cyan accent - secondary highlights, info states
    pub const CYAN: Color = Color::from_rgb(0.0, 0.878, 1.0); // #00e0ff - Electric cyan
    /// Cyan muted - subtle secondary accents
    pub const CYAN_MUTED: Color = Color::from_rgb(0.1, 0.314, 0.4); // #1a5066

    // === TEXT HIERARCHY (High contrast for readability) ===
    /// Primary text - headlines, important content
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.961, 0.969, 0.988); // #f5f7fc - Near white
    /// Secondary text - body content, descriptions
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.749, 0.773, 0.831); // #bfc5d4 - Cool gray
    /// Muted text - captions, hints, placeholders
    pub const TEXT_MUTED: Color = Color::from_rgb(0.502, 0.537, 0.616); // #80899d - Dim
    /// Disabled text - inactive elements
    pub const TEXT_DISABLED: Color = Color::from_rgb(0.337, 0.361, 0.424); // #565c6c - Very dim

    // === SEMANTIC COLORS (Vibrant and clear) ===
    /// Success - bright green with cyber feel
    pub const SUCCESS: Color = Color::from_rgb(0.2, 0.922, 0.557); // #33eb8e - Electric green
    /// Success muted - background for success states
    pub const SUCCESS_MUTED: Color = Color::from_rgb(0.1, 0.314, 0.2); // #1a5033
    /// Error - hot pink/red for immediate attention
    pub const ERROR: Color = Color::from_rgb(1.0, 0.314, 0.467); // #ff5077 - Hot pink
    /// Error muted - background for error states  
    pub const ERROR_MUTED: Color = Color::from_rgb(0.357, 0.129, 0.18); // #5b212e
    /// Warning - bright amber
    pub const WARNING: Color = Color::from_rgb(1.0, 0.757, 0.027); // #ffc107 - Amber
    /// Warning muted - background for warning states
    pub const WARNING_MUTED: Color = Color::from_rgb(0.357, 0.271, 0.071); // #5b4512
    /// Info - cyan blue
    pub const INFO: Color = Color::from_rgb(0.235, 0.729, 1.0); // #3cbaff - Sky blue

    // === BORDERS & STRUCTURE ===
    /// Primary border - visible element separation
    pub const BORDER: Color = Color::from_rgb(0.184, 0.212, 0.278); // #2f3647 - Subtle blue
    /// Light border - hover/focus emphasis
    pub const BORDER_LIGHT: Color = Color::from_rgb(0.243, 0.278, 0.361); // #3e475c
    /// Subtle border - minimal separation
    pub const BORDER_SUBTLE: Color = Color::from_rgb(0.122, 0.141, 0.184); // #1f242f
    /// Divider - section separators
    pub const DIVIDER: Color = Color::from_rgb(0.098, 0.114, 0.149); // #191d26
    /// Glow border - for focused/active elements
    pub const BORDER_GLOW: Color = Color::from_rgba(1.0, 0.784, 0.0, 0.5); // Gold glow border
}

// ============================================================================
// DESIGN SYSTEM - Neumorphic & Modern
// ============================================================================

pub mod design {
    use super::colors;
    use iced::{Color, Shadow, Vector};

    // === CORNER RADIUS (Larger for tactile feel) ===
    pub const RADIUS_XS: f32 = 4.0;
    pub const RADIUS_SM: f32 = 8.0;
    pub const RADIUS_MD: f32 = 12.0;
    pub const RADIUS_LG: f32 = 16.0;
    pub const RADIUS_XL: f32 = 24.0;
    pub const RADIUS_FULL: f32 = 9999.0; // Pill shape

    // === SHADOWS (Neumorphic depth system) ===

    /// Subtle shadow for minimal elevation
    pub fn shadow_sm() -> Shadow {
        Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 4.0,
        }
    }

    /// Medium shadow for cards and elevated content
    pub fn shadow_md() -> Shadow {
        Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        }
    }

    /// Large shadow for modals and floating elements
    pub fn shadow_lg() -> Shadow {
        Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        }
    }

    /// Glow shadow for primary actions (gold glow)
    pub fn shadow_glow() -> Shadow {
        Shadow {
            color: colors::GOLD_GLOW,
            offset: Vector::new(0.0, 0.0),
            blur_radius: 16.0,
        }
    }

    /// Intense glow for hover states
    pub fn shadow_glow_intense() -> Shadow {
        Shadow {
            color: Color::from_rgba(1.0, 0.784, 0.0, 0.5),
            offset: Vector::new(0.0, 0.0),
            blur_radius: 24.0,
        }
    }

    /// Error glow for danger buttons
    pub fn shadow_error_glow() -> Shadow {
        Shadow {
            color: Color::from_rgba(1.0, 0.314, 0.467, 0.4),
            offset: Vector::new(0.0, 0.0),
            blur_radius: 16.0,
        }
    }

    /// Success glow
    pub fn shadow_success_glow() -> Shadow {
        Shadow {
            color: Color::from_rgba(0.2, 0.922, 0.557, 0.3),
            offset: Vector::new(0.0, 0.0),
            blur_radius: 12.0,
        }
    }
}

// ============================================================================
// TYPOGRAPHY - Bold Hierarchy
// ============================================================================

pub mod typography {
    // === FONT SIZES (Bolder scale for impact) ===
    /// Display - hero headlines, splash screens
    pub const DISPLAY: f32 = 32.0;
    /// Title - page/section titles
    pub const TITLE: f32 = 24.0;
    /// Heading - subsection headers
    pub const HEADING: f32 = 20.0;
    /// Subheading - card titles, labels
    pub const SUBHEADING: f32 = 16.0;
    /// Body - main content text
    pub const BODY: f32 = 14.0;
    /// Caption - secondary info, timestamps
    pub const CAPTION: f32 = 12.0;
    /// Micro - badges, tiny labels
    pub const MICRO: f32 = 10.0;
}

// ============================================================================
// SPACING - Generous & Breathable
// ============================================================================

pub mod spacing {
    /// Micro - tight spacing, inline elements
    pub const MICRO: f32 = 2.0;
    /// Extra small - compact padding
    pub const XS: f32 = 4.0;
    /// Small - button padding, tight gaps
    pub const SM: f32 = 8.0;
    /// Medium - standard content spacing
    pub const MD: f32 = 12.0;
    /// Large - section padding
    pub const LG: f32 = 16.0;
    /// Extra large - major section gaps
    pub const XL: f32 = 24.0;
    /// 2XL - page margins, modal padding
    pub const XXL: f32 = 32.0;
    /// 3XL - hero sections
    pub const XXXL: f32 = 48.0;
}

// ============================================================================
// THEME CREATION
// ============================================================================

/// Create the custom iced theme with 2026 cyber dark palette
pub fn create_theme() -> Theme {
    Theme::custom(
        "Cyber Gold 2026".to_string(),
        iced::theme::Palette {
            background: colors::BG_BASE,
            text: colors::TEXT_PRIMARY,
            primary: colors::GOLD,
            success: colors::SUCCESS,
            warning: colors::WARNING,
            danger: colors::ERROR,
        },
    )
}

// ============================================================================
// BUTTON STYLES - Tactile & Glowing
// ============================================================================

/// Primary button - main CTA with gold glow effect
pub fn button_primary(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(colors::GOLD.into()),
        text_color: colors::BG_VOID,
        border: Border::default().rounded(design::RADIUS_MD),
        shadow: design::shadow_glow(),
        snap: true,
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(colors::GOLD_LIGHT.into()),
            shadow: design::shadow_glow_intense(),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(colors::GOLD_DARK.into()),
            shadow: design::shadow_sm(),
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

/// Secondary button - outlined style with subtle depth
pub fn button_secondary(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(colors::BG_ELEVATED.into()),
        text_color: colors::TEXT_PRIMARY,
        border: Border::default()
            .rounded(design::RADIUS_MD)
            .color(colors::BORDER)
            .width(1.5),
        shadow: design::shadow_sm(),
        snap: true,
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(colors::BG_HOVER.into()),
            text_color: colors::GOLD_LIGHT,
            border: Border::default()
                .rounded(design::RADIUS_MD)
                .color(colors::GOLD_MUTED)
                .width(1.5),
            shadow: design::shadow_md(),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(colors::BG_SELECTED.into()),
            text_color: colors::GOLD,
            border: Border::default()
                .rounded(design::RADIUS_MD)
                .color(colors::GOLD_DARK)
                .width(1.5),
            shadow: Shadow::default(),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(colors::BG_PANEL.into()),
            text_color: colors::TEXT_DISABLED,
            border: Border::default()
                .rounded(design::RADIUS_MD)
                .color(colors::BORDER_SUBTLE)
                .width(1.0),
            shadow: Shadow::default(),
            ..base
        },
    }
}

/// Danger button - hot pink with error glow
pub fn button_danger(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(colors::ERROR.into()),
        text_color: colors::BG_VOID,
        border: Border::default().rounded(design::RADIUS_MD),
        shadow: design::shadow_error_glow(),
        snap: true,
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Color::from_rgb(1.0, 0.45, 0.58).into()), // Lighter pink
            shadow: Shadow {
                color: Color::from_rgba(1.0, 0.314, 0.467, 0.6),
                offset: Vector::new(0.0, 0.0),
                blur_radius: 24.0,
            },
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Color::from_rgb(0.85, 0.2, 0.35).into()), // Darker pink
            shadow: design::shadow_sm(),
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

/// Success button - electric green glow
pub fn button_success(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(colors::SUCCESS.into()),
        text_color: colors::BG_VOID,
        border: Border::default().rounded(design::RADIUS_MD),
        shadow: design::shadow_success_glow(),
        snap: true,
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Color::from_rgb(0.35, 0.96, 0.65).into()),
            shadow: Shadow {
                color: Color::from_rgba(0.2, 0.922, 0.557, 0.5),
                offset: Vector::new(0.0, 0.0),
                blur_radius: 20.0,
            },
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Color::from_rgb(0.15, 0.75, 0.45).into()),
            shadow: design::shadow_sm(),
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

/// Ghost/frameless button - minimal, text-only feel
pub fn button_frameless(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: None,
        text_color: colors::TEXT_SECONDARY,
        border: Border::default().rounded(design::RADIUS_SM),
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

/// Icon button - compact, for toolbar actions
pub fn button_icon(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: None,
        text_color: colors::TEXT_MUTED,
        border: Border::default().rounded(design::RADIUS_SM),
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

// ============================================================================
// CONTAINER STYLES - Neumorphic Depth
// ============================================================================

/// Main panel container - sidebar and content areas
pub fn container_panel(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(colors::BG_PANEL.into()),
        border: Border::default().color(colors::BORDER_SUBTLE).width(1.0),
        shadow: Shadow::default(),
        text_color: None,
        snap: true,
    }
}

/// Elevated card - neumorphic raised surface
pub fn container_elevated(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(colors::BG_ELEVATED.into()),
        border: Border::default()
            .rounded(design::RADIUS_LG)
            .color(colors::BORDER)
            .width(1.0),
        shadow: design::shadow_md(),
        text_color: None,
        snap: true,
    }
}

/// Surface container - interactive content background
pub fn container_surface(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(colors::BG_SURFACE.into()),
        border: Border::default()
            .rounded(design::RADIUS_MD)
            .color(colors::BORDER_SUBTLE)
            .width(1.0),
        shadow: design::shadow_sm(),
        text_color: None,
        snap: true,
    }
}

/// Inset container - recessed appearance (neumorphic inset)
pub fn container_inset(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(colors::BG_BASE.into()),
        border: Border::default()
            .rounded(design::RADIUS_MD)
            .color(colors::BORDER_SUBTLE)
            .width(1.0),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            offset: Vector::new(0.0, -2.0), // Inset shadow
            blur_radius: 4.0,
        },
        text_color: None,
        snap: true,
    }
}

/// Modal backdrop - frosted dark overlay
pub fn container_modal_backdrop(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgba(0.02, 0.024, 0.035, 0.9).into()),
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: None,
        snap: true,
    }
}

/// Modal dialog - floating card with strong shadow
pub fn container_modal(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(colors::BG_PANEL.into()),
        border: Border::default()
            .rounded(design::RADIUS_XL)
            .color(colors::BORDER_LIGHT)
            .width(1.0),
        shadow: design::shadow_lg(),
        text_color: None,
        snap: true,
    }
}

/// Toolbar container - top menu bar
pub fn container_toolbar(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(colors::BG_ELEVATED.into()),
        border: Border::default().color(colors::BORDER_SUBTLE).width(1.0),
        shadow: design::shadow_sm(),
        text_color: None,
        snap: true,
    }
}

/// Status bar container - bottom info bar
pub fn container_status_bar(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(colors::BG_VOID.into()),
        border: Border::default().color(colors::BORDER_SUBTLE).width(1.0),
        shadow: Shadow::default(),
        text_color: None,
        snap: true,
    }
}

/// Success highlight container
pub fn container_success(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(colors::SUCCESS_MUTED.into()),
        border: Border::default()
            .rounded(design::RADIUS_MD)
            .color(colors::SUCCESS)
            .width(1.5),
        shadow: design::shadow_success_glow(),
        text_color: None,
        snap: true,
    }
}

/// Error highlight container
pub fn container_error(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(colors::ERROR_MUTED.into()),
        border: Border::default()
            .rounded(design::RADIUS_MD)
            .color(colors::ERROR)
            .width(1.5),
        shadow: design::shadow_error_glow(),
        text_color: None,
        snap: true,
    }
}

/// Warning highlight container
pub fn container_warning(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(colors::WARNING_MUTED.into()),
        border: Border::default()
            .rounded(design::RADIUS_MD)
            .color(colors::WARNING)
            .width(1.5),
        shadow: Shadow {
            color: Color::from_rgba(1.0, 0.757, 0.027, 0.3),
            offset: Vector::new(0.0, 0.0),
            blur_radius: 12.0,
        },
        text_color: None,
        snap: true,
    }
}

/// Gold accent container - for featured items
pub fn container_accent(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(colors::GOLD_MUTED.into()),
        border: Border::default()
            .rounded(design::RADIUS_MD)
            .color(colors::GOLD_DARK)
            .width(1.5),
        shadow: design::shadow_glow(),
        text_color: None,
        snap: true,
    }
}

// ============================================================================
// TEXT INPUT STYLES
// ============================================================================

/// Modern text input with focus glow
pub fn text_input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let base = text_input::Style {
        background: colors::BG_SURFACE.into(),
        border: Border::default()
            .rounded(design::RADIUS_MD)
            .color(colors::BORDER)
            .width(1.5),
        icon: colors::TEXT_MUTED,
        placeholder: colors::TEXT_MUTED,
        value: colors::TEXT_PRIMARY,
        selection: colors::GOLD_MUTED,
    };

    match status {
        text_input::Status::Active => base,
        text_input::Status::Hovered => text_input::Style {
            border: Border::default()
                .rounded(design::RADIUS_MD)
                .color(colors::BORDER_LIGHT)
                .width(1.5),
            ..base
        },
        text_input::Status::Focused { is_hovered: _ } => text_input::Style {
            border: Border::default()
                .rounded(design::RADIUS_MD)
                .color(colors::GOLD)
                .width(2.0),
            ..base
        },
        text_input::Status::Disabled => text_input::Style {
            background: colors::BG_PANEL.into(),
            value: colors::TEXT_DISABLED,
            border: Border::default()
                .rounded(design::RADIUS_MD)
                .color(colors::BORDER_SUBTLE)
                .width(1.0),
            ..base
        },
    }
}

// ============================================================================
// SCROLLABLE STYLES
// ============================================================================

/// Custom scrollbar with gold accent on interaction
pub fn scrollable_style(_theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let scroller_default = scrollable::Scroller {
        background: colors::BORDER.into(),
        border: Border::default().rounded(design::RADIUS_XS),
    };

    let scroller_hovered = scrollable::Scroller {
        background: colors::GOLD_MUTED.into(),
        border: Border::default().rounded(design::RADIUS_XS),
    };

    let scroller_dragged = scrollable::Scroller {
        background: colors::GOLD.into(),
        border: Border::default().rounded(design::RADIUS_XS),
    };

    let rail_default = scrollable::Rail {
        background: Some(colors::BG_ELEVATED.into()),
        border: Border::default().rounded(design::RADIUS_XS),
        scroller: scroller_default,
    };

    let rail_hovered = scrollable::Rail {
        background: Some(colors::BG_SURFACE.into()),
        border: Border::default().rounded(design::RADIUS_XS),
        scroller: scroller_hovered,
    };

    let rail_dragged = scrollable::Rail {
        background: Some(colors::BG_SURFACE.into()),
        border: Border::default().rounded(design::RADIUS_XS),
        scroller: scroller_dragged,
    };

    let auto_scroll = scrollable::AutoScroll {
        background: colors::BG_SURFACE.into(),
        border: Border::default().rounded(design::RADIUS_SM),
        shadow: design::shadow_sm(),
        icon: colors::TEXT_MUTED,
    };

    match status {
        scrollable::Status::Active { .. } => scrollable::Style {
            container: container::Style::default(),
            vertical_rail: rail_default,
            horizontal_rail: rail_default,
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
                rail_default
            },
            horizontal_rail: if is_horizontal_scrollbar_hovered {
                rail_hovered
            } else {
                rail_default
            },
            gap: None,
            auto_scroll,
        },
        scrollable::Status::Dragged {
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
            ..
        } => scrollable::Style {
            container: container::Style::default(),
            vertical_rail: if is_vertical_scrollbar_dragged {
                rail_dragged
            } else {
                rail_default
            },
            horizontal_rail: if is_horizontal_scrollbar_dragged {
                rail_dragged
            } else {
                rail_default
            },
            gap: None,
            auto_scroll,
        },
    }
}
