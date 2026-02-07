//! WeakAuras update categories
//!
//! Based on WeakAuras2 source code, defines categories for selective updates
//! when importing auras that already exist.

use std::collections::HashSet;

/// Update categories matching WeakAuras' "Categories to Update" dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UpdateCategory {
    /// Aura name/ID
    Name,
    /// Display settings (catch-all for unmatched fields)
    Display,
    /// Trigger configuration
    Trigger,
    /// Load conditions
    Load,
    /// Actions (on show/hide/init)
    Action,
    /// Animations
    Animation,
    /// Conditions (dynamic state changes)
    Conditions,
    /// Author options (custom options)
    AuthorOptions,
    /// Group arrangement (grow, space, sort, etc.)
    Arrangement,
    /// Anchor/position settings
    Anchor,
    /// User configuration values
    UserConfig,
    /// Metadata (url, desc, version, etc.)
    Metadata,
}

impl UpdateCategory {
    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            UpdateCategory::Name => "Name",
            UpdateCategory::Display => "Display",
            UpdateCategory::Trigger => "Trigger",
            UpdateCategory::Load => "Load",
            UpdateCategory::Action => "Actions",
            UpdateCategory::Animation => "Animations",
            UpdateCategory::Conditions => "Conditions",
            UpdateCategory::AuthorOptions => "Author Options",
            UpdateCategory::Arrangement => "Arrangement",
            UpdateCategory::Anchor => "Anchor",
            UpdateCategory::UserConfig => "User Config",
            UpdateCategory::Metadata => "Metadata",
        }
    }

    /// Whether this category is enabled by default when importing
    pub fn default_enabled(&self) -> bool {
        match self {
            // These are OFF by default (preserve user customizations)
            UpdateCategory::Anchor => false,
            UpdateCategory::UserConfig => false,
            // Everything else is ON by default
            _ => true,
        }
    }

    /// Get all categories in display order
    pub fn all() -> Vec<UpdateCategory> {
        vec![
            UpdateCategory::Name,
            UpdateCategory::Display,
            UpdateCategory::Trigger,
            UpdateCategory::Load,
            UpdateCategory::Action,
            UpdateCategory::Animation,
            UpdateCategory::Conditions,
            UpdateCategory::AuthorOptions,
            UpdateCategory::Arrangement,
            UpdateCategory::Anchor,
            UpdateCategory::UserConfig,
            UpdateCategory::Metadata,
        ]
    }

    /// Get categories enabled by default
    pub fn defaults() -> HashSet<UpdateCategory> {
        Self::all()
            .into_iter()
            .filter(|c| c.default_enabled())
            .collect()
    }
}

/// Maps field names to their category
pub struct CategoryMapper;

impl CategoryMapper {
    /// Fields that belong to the Trigger category
    const TRIGGER_FIELDS: &'static [&'static str] = &["triggers"];

    /// Fields that belong to the Load category
    const LOAD_FIELDS: &'static [&'static str] = &["load"];

    /// Fields that belong to the Action category
    const ACTION_FIELDS: &'static [&'static str] = &["actions"];

    /// Fields that belong to the Animation category
    const ANIMATION_FIELDS: &'static [&'static str] = &["animation"];

    /// Fields that belong to the Conditions category
    const CONDITIONS_FIELDS: &'static [&'static str] = &["conditions"];

    /// Fields that belong to the AuthorOptions category
    const AUTHOR_OPTIONS_FIELDS: &'static [&'static str] = &["authorOptions"];

    /// Fields that belong to the Arrangement category (for groups)
    const ARRANGEMENT_FIELDS: &'static [&'static str] = &[
        "grow",
        "space",
        "stagger",
        "sort",
        "sortHybridTable",
        "radius",
        "align",
        "rotation",
        "constantFactor",
        "gridType",
        "gridWidth",
        "rowSpace",
        "columnSpace",
        "fullCircle",
        "arcLength",
        "animate",
        "useLimit",
        "limit",
        "centerType",
    ];

    /// Fields that belong to the Anchor category (position/size)
    const ANCHOR_FIELDS: &'static [&'static str] = &[
        "xOffset",
        "yOffset",
        "selfPoint",
        "anchorPoint",
        "anchorFrameType",
        "anchorFrameFrame",
        "anchorFrameParent",
        "frameStrata",
        "width",
        "height",
        "scale",
        "fontSize",
    ];

    /// Fields that belong to the UserConfig category
    const USER_CONFIG_FIELDS: &'static [&'static str] = &["config"];

    /// Fields that belong to the Metadata category
    const METADATA_FIELDS: &'static [&'static str] =
        &["url", "desc", "version", "semver", "wagoID"];

    /// Fields that belong to the Name category
    const NAME_FIELDS: &'static [&'static str] = &["id"];

    /// Internal fields that should be excluded from comparison and not categorized
    const INTERNAL_FIELDS: &'static [&'static str] = &[
        "uid",
        "internalVersion",
        "tocversion",
        "parent",
        "controlledChildren",
        "source",
        "preferToUpdate",
        "skipWagoUpdate",
        "ignoreWagoUpdate",
    ];

    /// Check if a field is internal (should be excluded from comparison)
    pub fn is_internal_field(field: &str) -> bool {
        Self::INTERNAL_FIELDS.contains(&field)
    }

    /// Get the category for a field name
    pub fn get_category(field: &str) -> UpdateCategory {
        if Self::NAME_FIELDS.contains(&field) {
            UpdateCategory::Name
        } else if Self::TRIGGER_FIELDS.contains(&field) {
            UpdateCategory::Trigger
        } else if Self::LOAD_FIELDS.contains(&field) {
            UpdateCategory::Load
        } else if Self::ACTION_FIELDS.contains(&field) {
            UpdateCategory::Action
        } else if Self::ANIMATION_FIELDS.contains(&field) {
            UpdateCategory::Animation
        } else if Self::CONDITIONS_FIELDS.contains(&field) {
            UpdateCategory::Conditions
        } else if Self::AUTHOR_OPTIONS_FIELDS.contains(&field) {
            UpdateCategory::AuthorOptions
        } else if Self::ARRANGEMENT_FIELDS.contains(&field) {
            UpdateCategory::Arrangement
        } else if Self::ANCHOR_FIELDS.contains(&field) {
            UpdateCategory::Anchor
        } else if Self::USER_CONFIG_FIELDS.contains(&field) {
            UpdateCategory::UserConfig
        } else if Self::METADATA_FIELDS.contains(&field) {
            UpdateCategory::Metadata
        } else {
            // Everything else goes to Display (catch-all)
            UpdateCategory::Display
        }
    }

    /// Get all fields for a category
    pub fn get_fields(category: UpdateCategory) -> &'static [&'static str] {
        match category {
            UpdateCategory::Name => Self::NAME_FIELDS,
            UpdateCategory::Trigger => Self::TRIGGER_FIELDS,
            UpdateCategory::Load => Self::LOAD_FIELDS,
            UpdateCategory::Action => Self::ACTION_FIELDS,
            UpdateCategory::Animation => Self::ANIMATION_FIELDS,
            UpdateCategory::Conditions => Self::CONDITIONS_FIELDS,
            UpdateCategory::AuthorOptions => Self::AUTHOR_OPTIONS_FIELDS,
            UpdateCategory::Arrangement => Self::ARRANGEMENT_FIELDS,
            UpdateCategory::Anchor => Self::ANCHOR_FIELDS,
            UpdateCategory::UserConfig => Self::USER_CONFIG_FIELDS,
            UpdateCategory::Metadata => Self::METADATA_FIELDS,
            UpdateCategory::Display => &[], // Catch-all, no specific fields
        }
    }
}
