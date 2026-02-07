//! Tests for update category mapping.

use weakaura_mass_import::categories::{CategoryMapper, UpdateCategory};

#[test]
fn test_category_mapping() {
    assert_eq!(
        CategoryMapper::get_category("triggers"),
        UpdateCategory::Trigger
    );
    assert_eq!(CategoryMapper::get_category("load"), UpdateCategory::Load);
    assert_eq!(
        CategoryMapper::get_category("xOffset"),
        UpdateCategory::Anchor
    );
    assert_eq!(
        CategoryMapper::get_category("url"),
        UpdateCategory::Metadata
    );
    assert_eq!(
        CategoryMapper::get_category("someRandomField"),
        UpdateCategory::Display
    );
}

#[test]
fn test_internal_fields() {
    assert!(CategoryMapper::is_internal_field("uid"));
    assert!(CategoryMapper::is_internal_field("internalVersion"));
    assert!(!CategoryMapper::is_internal_field("triggers"));
}

#[test]
fn test_default_enabled() {
    assert!(UpdateCategory::Trigger.default_enabled());
    assert!(!UpdateCategory::Anchor.default_enabled());
    assert!(!UpdateCategory::UserConfig.default_enabled());
}
