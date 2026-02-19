//! Tests for SavedVariables management and hierarchy preservation.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use weakauras_mass_import::categories::UpdateCategory;
use weakauras_mass_import::decoder::{LuaValue, WeakAura, WeakAuraDecoder};
use weakauras_mass_import::saved_variables::SavedVariablesManager;
use weakauras_mass_import::saved_variables::{ConflictAction, ConflictResolution};

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

/// Helper: create a simple non-group aura table.
fn make_aura(id: &str, parent: Option<&str>) -> LuaValue {
    let mut table = HashMap::new();
    table.insert("id".to_string(), LuaValue::String(id.to_string()));
    table.insert(
        "regionType".to_string(),
        LuaValue::String("icon".to_string()),
    );
    if let Some(p) = parent {
        table.insert("parent".to_string(), LuaValue::String(p.to_string()));
    }
    LuaValue::Table(table)
}

/// Helper: create a group aura table with controlledChildren.
fn make_group(id: &str, parent: Option<&str>, children: &[&str]) -> LuaValue {
    let mut table = HashMap::new();
    table.insert("id".to_string(), LuaValue::String(id.to_string()));
    table.insert(
        "regionType".to_string(),
        LuaValue::String("group".to_string()),
    );
    if let Some(p) = parent {
        table.insert("parent".to_string(), LuaValue::String(p.to_string()));
    }
    let controlled: Vec<LuaValue> = children
        .iter()
        .map(|c| LuaValue::String(c.to_string()))
        .collect();
    table.insert(
        "controlledChildren".to_string(),
        LuaValue::Array(controlled),
    );
    LuaValue::Table(table)
}

/// Helper: build a manager with pre-populated displays (no file needed).
fn manager_with_displays(displays: HashMap<String, LuaValue>) -> SavedVariablesManager {
    let mut mgr = SavedVariablesManager::new(PathBuf::from("test_remove.lua"));
    mgr.displays = displays;
    mgr
}

fn make_group_with_fields(
    id: &str,
    parent: Option<&str>,
    children: &[&str],
    custom: &str,
) -> LuaValue {
    let mut table = HashMap::new();
    table.insert("id".to_string(), LuaValue::String(id.to_string()));
    table.insert(
        "regionType".to_string(),
        LuaValue::String("group".to_string()),
    );
    table.insert("custom".to_string(), LuaValue::String(custom.to_string()));
    if let Some(p) = parent {
        table.insert("parent".to_string(), LuaValue::String(p.to_string()));
    }
    let controlled: Vec<LuaValue> = children
        .iter()
        .map(|c| LuaValue::String(c.to_string()))
        .collect();
    table.insert(
        "controlledChildren".to_string(),
        LuaValue::Array(controlled),
    );
    LuaValue::Table(table)
}

fn make_aura_with_custom(id: &str, parent: Option<&str>, custom: &str) -> LuaValue {
    let mut table = HashMap::new();
    table.insert("id".to_string(), LuaValue::String(id.to_string()));
    table.insert(
        "regionType".to_string(),
        LuaValue::String("icon".to_string()),
    );
    table.insert("custom".to_string(), LuaValue::String(custom.to_string()));
    if let Some(p) = parent {
        table.insert("parent".to_string(), LuaValue::String(p.to_string()));
    }
    LuaValue::Table(table)
}

#[test]
fn test_remove_standalone_aura() {
    let mut displays = HashMap::new();
    displays.insert("Aura1".to_string(), make_aura("Aura1", None));
    displays.insert("Aura2".to_string(), make_aura("Aura2", None));
    displays.insert("Aura3".to_string(), make_aura("Aura3", None));

    let mut mgr = manager_with_displays(displays);
    assert_eq!(mgr.displays.len(), 3);

    let removed = mgr.remove_auras(&["Aura2".to_string()]);

    assert_eq!(removed, vec!["Aura2".to_string()]);
    assert_eq!(mgr.displays.len(), 2);
    assert!(mgr.displays.contains_key("Aura1"));
    assert!(!mgr.displays.contains_key("Aura2"));
    assert!(mgr.displays.contains_key("Aura3"));
}

#[test]
fn test_remove_group_removes_all_descendants() {
    // Group hierarchy: RootGroup -> [ChildA, SubGroup -> [GrandchildX]]
    let mut displays = HashMap::new();
    displays.insert(
        "RootGroup".to_string(),
        make_group("RootGroup", None, &["ChildA", "SubGroup"]),
    );
    displays.insert("ChildA".to_string(), make_aura("ChildA", Some("RootGroup")));
    displays.insert(
        "SubGroup".to_string(),
        make_group("SubGroup", Some("RootGroup"), &["GrandchildX"]),
    );
    displays.insert(
        "GrandchildX".to_string(),
        make_aura("GrandchildX", Some("SubGroup")),
    );
    displays.insert("Standalone".to_string(), make_aura("Standalone", None));

    let mut mgr = manager_with_displays(displays);
    assert_eq!(mgr.displays.len(), 5);

    let mut removed = mgr.remove_auras(&["RootGroup".to_string()]);
    removed.sort();

    assert_eq!(
        removed,
        vec![
            "ChildA".to_string(),
            "GrandchildX".to_string(),
            "RootGroup".to_string(),
            "SubGroup".to_string(),
        ]
    );
    // Only the standalone should remain
    assert_eq!(mgr.displays.len(), 1);
    assert!(mgr.displays.contains_key("Standalone"));
}

#[test]
fn test_remove_child_updates_parent_controlled_children() {
    // Group with two children; remove one, verify parent's controlledChildren updated
    let mut displays = HashMap::new();
    displays.insert(
        "MyGroup".to_string(),
        make_group("MyGroup", None, &["Child1", "Child2"]),
    );
    displays.insert("Child1".to_string(), make_aura("Child1", Some("MyGroup")));
    displays.insert("Child2".to_string(), make_aura("Child2", Some("MyGroup")));

    let mut mgr = manager_with_displays(displays);

    let removed = mgr.remove_auras(&["Child1".to_string()]);
    assert_eq!(removed, vec!["Child1".to_string()]);
    assert_eq!(mgr.displays.len(), 2);

    // Verify parent's controlledChildren now only has Child2
    let group = mgr.displays.get("MyGroup").unwrap();
    let table = group.as_table().unwrap();
    let controlled = table.get("controlledChildren").unwrap();
    let arr = controlled.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0], LuaValue::String("Child2".to_string()));
}

#[test]
fn test_remove_nonexistent_returns_empty() {
    let mut displays = HashMap::new();
    displays.insert("Aura1".to_string(), make_aura("Aura1", None));

    let mut mgr = manager_with_displays(displays);

    let removed = mgr.remove_auras(&["DoesNotExist".to_string()]);
    assert!(removed.is_empty());
    assert_eq!(mgr.displays.len(), 1);
}

#[test]
fn test_replace_group_removes_stale_children_and_updates_descendants() {
    let mut displays = HashMap::new();
    displays.insert(
        "Root".to_string(),
        make_group_with_fields("Root", None, &["ChildA", "ChildB", "StaleChild"], "old"),
    );
    displays.insert(
        "ChildA".to_string(),
        make_aura_with_custom("ChildA", Some("Root"), "oldA"),
    );
    displays.insert(
        "ChildB".to_string(),
        make_aura_with_custom("ChildB", Some("Root"), "oldB"),
    );
    displays.insert(
        "StaleChild".to_string(),
        make_aura_with_custom("StaleChild", Some("Root"), "stale"),
    );

    let mut mgr = manager_with_displays(displays);

    let root_data = make_group_with_fields("Root", None, &["ChildA", "ChildC"], "new");
    let child_a_data = make_aura_with_custom("ChildA", Some("Root"), "newA");
    let child_c_data = make_aura_with_custom("ChildC", Some("Root"), "newC");

    let incoming = WeakAura {
        id: "Root".to_string(),
        uid: None,
        region_type: Some("group".to_string()),
        is_group: true,
        children: vec!["ChildA".to_string(), "ChildC".to_string()],
        data: root_data,
        child_data: vec![child_a_data, child_c_data],
        original_string: String::new(),
        encoding_version: 2,
    };

    let conflicts = mgr.detect_conflicts(&[incoming]);
    let resolutions = vec![ConflictResolution {
        aura_id: "Root".to_string(),
        action: ConflictAction::ReplaceAll,
        categories_to_update: Default::default(),
    }];

    let _result = mgr.apply_resolutions(&conflicts, &resolutions);

    assert!(mgr.displays.contains_key("Root"));
    assert!(mgr.displays.contains_key("ChildA"));
    assert!(mgr.displays.contains_key("ChildC"));
    assert!(!mgr.displays.contains_key("ChildB"));
    assert!(!mgr.displays.contains_key("StaleChild"));

    let child_a = mgr.displays.get("ChildA").unwrap();
    let child_a_table = child_a.as_table().unwrap();
    assert_eq!(
        child_a_table.get("custom"),
        Some(&LuaValue::String("newA".to_string()))
    );

    let root = mgr.displays.get("Root").unwrap();
    let root_table = root.as_table().unwrap();
    let controlled = root_table.get("controlledChildren").unwrap();
    let controlled_arr = controlled.as_array().unwrap();
    let controlled_ids: Vec<String> = controlled_arr
        .iter()
        .filter_map(|value| {
            if let LuaValue::String(s) = value {
                Some(s.clone())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        controlled_ids,
        vec!["ChildA".to_string(), "ChildC".to_string()]
    );
}

#[test]
fn test_update_selected_arrangement_prunes_missing_children() {
    let mut displays = HashMap::new();
    displays.insert(
        "Root".to_string(),
        make_group_with_fields("Root", None, &["ChildA", "StaleChild"], "old"),
    );
    displays.insert(
        "ChildA".to_string(),
        make_aura_with_custom("ChildA", Some("Root"), "oldA"),
    );
    displays.insert(
        "StaleChild".to_string(),
        make_aura_with_custom("StaleChild", Some("Root"), "stale"),
    );

    let mut mgr = manager_with_displays(displays);

    let root_data = make_group_with_fields("Root", None, &["ChildA"], "new");
    let child_a_data = make_aura_with_custom("ChildA", Some("Root"), "oldA");

    let incoming = WeakAura {
        id: "Root".to_string(),
        uid: None,
        region_type: Some("group".to_string()),
        is_group: true,
        children: vec!["ChildA".to_string()],
        data: root_data,
        child_data: vec![child_a_data],
        original_string: String::new(),
        encoding_version: 2,
    };

    let conflicts = mgr.detect_conflicts(&[incoming]);
    let mut categories = HashSet::new();
    categories.insert(UpdateCategory::Arrangement);
    let resolutions = vec![ConflictResolution {
        aura_id: "Root".to_string(),
        action: ConflictAction::UpdateSelected,
        categories_to_update: categories,
    }];

    let _result = mgr.apply_resolutions(&conflicts, &resolutions);

    assert!(mgr.displays.contains_key("Root"));
    assert!(mgr.displays.contains_key("ChildA"));
    assert!(!mgr.displays.contains_key("StaleChild"));
}
