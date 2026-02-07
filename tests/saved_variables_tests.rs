//! Tests for SavedVariables management and hierarchy preservation.

use std::path::PathBuf;

use weakaura_mass_import::decoder::{LuaValue, WeakAuraDecoder};
use weakaura_mass_import::saved_variables::SavedVariablesManager;

/// Helper: decode the Hunter import string and run it through add_auras,
/// then verify the parent-child hierarchy is correctly preserved.
#[test]
fn test_add_auras_preserves_hierarchy() {
    let import_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Hunter (QHD).txt");
    if !import_path.exists() {
        eprintln!("Skipping test: Hunter (QHD).txt not found");
        return;
    }

    let input = std::fs::read_to_string(&import_path).unwrap();
    let aura = WeakAuraDecoder::decode(input.trim()).unwrap();

    assert_eq!(aura.id, "[Merfin] Hunter (Quad HD)");
    assert!(aura.is_group);
    assert_eq!(
        aura.children.len(),
        10,
        "Root should have 10 direct children"
    );
    assert_eq!(
        aura.child_data.len(),
        56,
        "Should have 56 total descendants"
    );

    // Create a fresh manager and add the aura
    let mut manager = SavedVariablesManager::new(PathBuf::from("test_output.lua"));
    let _result = manager.add_auras(&[aura]).unwrap();

    // Verify the root group's controlledChildren has only 10 direct children
    let root = manager.displays.get("[Merfin] Hunter (Quad HD)").unwrap();
    let root_table = root.as_table().unwrap();
    let controlled = root_table.get("controlledChildren").unwrap();
    let controlled_arr = controlled.as_array().unwrap();
    assert_eq!(
        controlled_arr.len(),
        10,
        "Root group should have exactly 10 direct children, got {}",
        controlled_arr.len()
    );

    // Verify [Hunter] Buffs is a subgroup with its own controlledChildren
    let buffs = manager.displays.get("[Hunter] Buffs").unwrap();
    let buffs_table = buffs.as_table().unwrap();
    let buffs_controlled = buffs_table.get("controlledChildren").unwrap();
    let buffs_children = buffs_controlled.as_array().unwrap();
    assert!(
        !buffs_children.is_empty(),
        "[Hunter] Buffs should have controlledChildren"
    );

    // Verify [Hunter] [Buff] Class has parent = "[Hunter] Buffs", NOT the root
    let buff_class = manager.displays.get("[Hunter] [Buff] Class").unwrap();
    let buff_class_table = buff_class.as_table().unwrap();
    let parent_val = buff_class_table.get("parent").unwrap();
    if let LuaValue::String(parent_id) = parent_val {
        assert_eq!(
            parent_id, "[Hunter] Buffs",
            "[Hunter] [Buff] Class should have parent = [Hunter] Buffs, got {}",
            parent_id
        );
    } else {
        panic!("parent field should be a string");
    }

    // Verify [Hunter] Debuffs has its own controlledChildren (only direct child)
    let debuffs = manager.displays.get("[Hunter] Debuffs").unwrap();
    let debuffs_table = debuffs.as_table().unwrap();
    let debuffs_controlled = debuffs_table.get("controlledChildren").unwrap();
    let debuffs_children = debuffs_controlled.as_array().unwrap();
    assert_eq!(
        debuffs_children.len(),
        1,
        "[Hunter] Debuffs should have exactly 1 child (the icon), got {}",
        debuffs_children.len()
    );

    // Verify total aura count: 1 root + 56 children = 57
    assert_eq!(
        manager.displays.len(),
        57,
        "Should have 57 total auras (1 root + 56 children)"
    );

    // Count how many auras have parent = root
    let root_children_count = manager
        .displays
        .values()
        .filter(|v| {
            v.as_table()
                .and_then(|t| t.get("parent"))
                .map(|p| matches!(p, LuaValue::String(s) if s == "[Merfin] Hunter (Quad HD)"))
                .unwrap_or(false)
        })
        .count();

    assert_eq!(
        root_children_count, 10,
        "Exactly 10 auras should have the root as their parent, got {}",
        root_children_count
    );

    println!("All hierarchy assertions passed!");
    println!(
        "  Root controlledChildren: {} (expected 10)",
        controlled_arr.len()
    );
    println!("  [Hunter] Buffs children: {}", buffs_children.len());
    println!("  [Hunter] Debuffs children: {}", debuffs_children.len());
    println!(
        "  Auras with root as parent: {} (expected 10)",
        root_children_count
    );
    println!("  Total displays: {}", manager.displays.len());
}
