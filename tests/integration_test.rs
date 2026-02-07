//! Integration tests for the full WeakAura decode -> import -> serialize pipeline.
//!
//! Uses real WeakAura import strings to test the complete flow.

use std::collections::HashMap;

use weakaura_mass_import::decoder::{LuaValue, WeakAuraDecoder};
use weakaura_mass_import::lua_parser::LuaParser;
use weakaura_mass_import::saved_variables::SavedVariablesManager;

/// Load a test WA string from a file
fn load_test_string(filename: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(filename);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read test file {}: {}", path.display(), e))
}

// ──────────────────────────────────────────────────
// Decode tests
// ──────────────────────────────────────────────────

#[test]
fn test_decode_merfin_anchors() {
    let input = load_test_string("[Merfin] Anchors_UI_QHD.txt");
    let result = WeakAuraDecoder::decode(&input);
    assert!(
        result.is_ok(),
        "Failed to decode Merfin Anchors: {:?}",
        result.err()
    );

    let aura = result.unwrap();
    assert_eq!(aura.id, "![Merfin] UI Anchors (QHD Display)");
    assert!(aura.is_group, "Should be detected as a group");
    assert!(
        !aura.children.is_empty(),
        "Group should have children (got 0)"
    );
    assert!(
        !aura.child_data.is_empty(),
        "Group should have child_data (got 0)"
    );
    assert_eq!(
        aura.children.len(),
        aura.child_data.len(),
        "children count should match child_data count"
    );
    assert_eq!(aura.encoding_version, 2);
}

#[test]
fn test_decode_hunter_qhd() {
    let input = load_test_string("Hunter (QHD).txt");
    let result = WeakAuraDecoder::decode(&input);
    assert!(
        result.is_ok(),
        "Failed to decode Hunter QHD: {:?}",
        result.err()
    );

    let aura = result.unwrap();
    assert_eq!(aura.id, "[Merfin] Hunter (Quad HD)");
    assert!(aura.is_group, "Should be detected as a group");
    assert!(!aura.children.is_empty(), "Group should have children");
    assert!(!aura.child_data.is_empty(), "Group should have child_data");
    assert_eq!(aura.encoding_version, 2);
}

// ──────────────────────────────────────────────────
// Metadata extraction tests
// ──────────────────────────────────────────────────

#[test]
fn test_metadata_extraction_handles_mixed_table_data() {
    let input = load_test_string("[Merfin] Anchors_UI_QHD.txt");
    let aura = WeakAuraDecoder::decode(&input).unwrap();

    // The parent data should be accessible via as_table()
    let parent_table = aura.data.as_table();
    assert!(
        parent_table.is_some(),
        "Parent data should be accessible via as_table()"
    );

    // Each child should also have accessible data
    for (i, child) in aura.child_data.iter().enumerate() {
        let child_table = child.as_table();
        assert!(
            child_table.is_some(),
            "Child {} data should be accessible via as_table()",
            i
        );

        // Each child should have an id
        if let Some(table) = child_table {
            assert!(
                table.contains_key("id"),
                "Child {} should have an 'id' field",
                i
            );
        }
    }
}

#[test]
fn test_children_have_triggers_as_mixed_table() {
    // Many WA children have triggers as MixedTable (array + hash parts)
    let input = load_test_string("[Merfin] Anchors_UI_QHD.txt");
    let aura = WeakAuraDecoder::decode(&input).unwrap();

    let mut found_mixed_triggers = false;
    for child in &aura.child_data {
        if let Some(table) = child.as_table() {
            if let Some(triggers) = table.get("triggers") {
                if matches!(triggers, LuaValue::MixedTable { .. }) {
                    found_mixed_triggers = true;
                    break;
                }
            }
        }
    }

    assert!(
        found_mixed_triggers,
        "At least one child should have triggers as a MixedTable"
    );
}

// ──────────────────────────────────────────────────
// SavedVariables round-trip tests
// ──────────────────────────────────────────────────

#[test]
fn test_parse_working_saved_variables() {
    let content = load_test_string("WeakAuras-working.lua");
    let result = LuaParser::parse(&content);
    assert!(
        result.is_ok(),
        "Failed to parse working SavedVariables: {:?}",
        result.err()
    );

    let saved = result.unwrap();
    assert!(
        !saved.displays.is_empty(),
        "Working SavedVariables should have displays"
    );
}

#[test]
fn test_parse_non_working_saved_variables() {
    let content = load_test_string("WeakAuras-non-working.lua");
    let result = LuaParser::parse(&content);
    assert!(
        result.is_ok(),
        "Failed to parse non-working SavedVariables: {:?}",
        result.err()
    );

    let saved = result.unwrap();
    assert!(
        !saved.displays.is_empty(),
        "Non-working SavedVariables should have displays"
    );
}

#[test]
fn test_saved_variables_round_trip_preserves_structure() {
    // Parse a real SavedVariables file, serialize it back, parse again,
    // and verify the structure is equivalent
    let content = load_test_string("WeakAuras-working.lua");
    let saved1 = LuaParser::parse(&content).unwrap();

    let display_count_1 = saved1.displays.len();

    // Create a manager and load
    let temp_path = std::env::temp_dir().join("weakauras_test_roundtrip.lua");
    let mut manager = SavedVariablesManager::new(temp_path.clone());
    manager.displays = saved1.displays;

    // Generate lua and write
    let generated = manager.generate_lua();
    std::fs::write(&temp_path, &generated).unwrap();

    // Parse the generated file
    let saved2 = LuaParser::parse(&generated).unwrap();

    assert_eq!(
        display_count_1,
        saved2.displays.len(),
        "Display count should be preserved after round-trip (expected {}, got {})",
        display_count_1,
        saved2.displays.len()
    );

    // Verify every display key is present
    for key in manager.displays.keys() {
        assert!(
            saved2.displays.contains_key(key),
            "Display '{}' should be preserved after round-trip",
            key
        );
    }

    // Cleanup
    let _ = std::fs::remove_file(&temp_path);
}

// ──────────────────────────────────────────────────
// Full pipeline: decode -> add to SV -> serialize -> re-parse
// ──────────────────────────────────────────────────

#[test]
fn test_full_import_pipeline_merfin_anchors() {
    let input = load_test_string("[Merfin] Anchors_UI_QHD.txt");
    let aura = WeakAuraDecoder::decode(&input).unwrap();

    // Create an empty manager
    let temp_path = std::env::temp_dir().join("weakauras_test_merfin.lua");
    let mut manager = SavedVariablesManager::new(temp_path.clone());

    // Add the aura
    let result = manager.add_auras(&[aura.clone()]).unwrap();

    // Verify all children were added
    assert!(!result.added.is_empty(), "Should have added some auras");
    assert!(
        result.added.contains(&aura.id),
        "Parent '{}' should be in added list",
        aura.id
    );
    for child_name in &aura.children {
        assert!(
            result.added.contains(child_name),
            "Child '{}' should be in added list",
            child_name
        );
    }

    // Verify parent has controlledChildren set
    let parent_data = manager.displays.get(&aura.id).unwrap();
    if let Some(table) = parent_data.as_table() {
        assert!(
            table.contains_key("controlledChildren"),
            "Parent should have controlledChildren after import"
        );
    } else {
        panic!("Parent data should be accessible as table");
    }

    // Verify children have parent field set
    for child_name in &aura.children {
        let child_data = manager.displays.get(child_name).unwrap();
        if let Some(table) = child_data.as_table() {
            assert!(
                table.contains_key("parent"),
                "Child '{}' should have 'parent' field set",
                child_name
            );
            if let Some(LuaValue::String(parent_id)) = table.get("parent") {
                assert_eq!(
                    parent_id, &aura.id,
                    "Child '{}' parent should be '{}'",
                    child_name, aura.id
                );
            }
        } else {
            panic!("Child '{}' data should be accessible as table", child_name);
        }
    }

    // Serialize to Lua and re-parse
    let generated = manager.generate_lua();
    let reparsed = LuaParser::parse(&generated);
    assert!(
        reparsed.is_ok(),
        "Generated Lua should be parseable: {:?}",
        reparsed.err()
    );

    let reparsed = reparsed.unwrap();
    assert_eq!(
        manager.displays.len(),
        reparsed.displays.len(),
        "Display count should survive serialization round-trip"
    );

    // Cleanup
    let _ = std::fs::remove_file(&temp_path);
}

#[test]
fn test_full_import_pipeline_hunter_qhd() {
    let input = load_test_string("Hunter (QHD).txt");
    let aura = WeakAuraDecoder::decode(&input).unwrap();

    let temp_path = std::env::temp_dir().join("weakauras_test_hunter.lua");
    let mut manager = SavedVariablesManager::new(temp_path.clone());

    let result = manager.add_auras(&[aura.clone()]).unwrap();

    assert!(result.added.contains(&aura.id), "Parent should be added");

    // Serialize and re-parse
    let generated = manager.generate_lua();
    let reparsed = LuaParser::parse(&generated);
    assert!(
        reparsed.is_ok(),
        "Generated Lua should be parseable: {:?}",
        reparsed.err()
    );

    let reparsed = reparsed.unwrap();
    assert_eq!(
        manager.displays.len(),
        reparsed.displays.len(),
        "Display count should survive serialization round-trip"
    );

    // Cleanup
    let _ = std::fs::remove_file(&temp_path);
}

// ──────────────────────────────────────────────────
// Import into existing SavedVariables (conflict detection)
// ──────────────────────────────────────────────────

#[test]
fn test_conflict_detection_with_existing_data() {
    let input = load_test_string("[Merfin] Anchors_UI_QHD.txt");
    let aura = WeakAuraDecoder::decode(&input).unwrap();

    let temp_path = std::env::temp_dir().join("weakauras_test_conflict.lua");
    let mut manager = SavedVariablesManager::new(temp_path.clone());

    // First import
    manager.add_auras(&[aura.clone()]).unwrap();

    // Second import of identical data - detect_conflicts should find NO new auras
    // but conflicts may be empty if data is identical (no changes detected)
    let conflicts = manager.detect_conflicts(&[aura.clone()]);

    assert!(
        conflicts.new_auras.is_empty(),
        "Second import should have no new auras (all already exist)"
    );

    // With identical data, conflicts list could be empty (no changes) or populated.
    // The important thing is that nothing is treated as "new".
    // The parent and children were already added, so total conflicts + unchanged should cover all.
    let _total_detected = conflicts.conflicts.len() + conflicts.new_auras.len();
    // At minimum, none should be new
    assert_eq!(
        conflicts.new_auras.len(),
        0,
        "No auras should be new on second import"
    );

    // Cleanup
    let _ = std::fs::remove_file(&temp_path);
}

// ──────────────────────────────────────────────────
// Lua serialization edge cases
// ──────────────────────────────────────────────────

#[test]
fn test_serialize_nan() {
    let value = LuaValue::Number(f64::NAN);
    let result = LuaParser::serialize(&value, 0);
    assert_eq!(result, "(0/0)", "NaN should serialize to (0/0)");
}

#[test]
fn test_serialize_infinity() {
    let pos_inf = LuaValue::Number(f64::INFINITY);
    let neg_inf = LuaValue::Number(f64::NEG_INFINITY);

    assert_eq!(
        LuaParser::serialize(&pos_inf, 0),
        "math.huge",
        "Positive infinity should serialize to math.huge"
    );
    assert_eq!(
        LuaParser::serialize(&neg_inf, 0),
        "-math.huge",
        "Negative infinity should serialize to -math.huge"
    );
}

#[test]
fn test_parse_hex_numbers() {
    // Hex numbers should be parsed correctly
    let input = r#"WeakAurasSaved = { displays = { ["test"] = {
        ["color"] = 0xFF00AA,
    } } }"#;

    let result = LuaParser::parse(input).unwrap();
    if let Some(LuaValue::Table(test)) = result.displays.get("test") {
        if let Some(LuaValue::Number(n)) = test.get("color") {
            assert_eq!(
                *n, 0xFF00AA as f64,
                "Hex number 0xFF00AA should parse to {} but got {}",
                0xFF00AA as f64, n
            );
        } else {
            panic!("color should be a Number");
        }
    } else {
        panic!("displays should have test key");
    }
}

// ──────────────────────────────────────────────────
// Deterministic output
// ──────────────────────────────────────────────────

#[test]
fn test_generate_lua_deterministic() {
    let temp_path = std::env::temp_dir().join("weakauras_test_deterministic.lua");
    let mut manager = SavedVariablesManager::new(temp_path.clone());

    // Insert displays in arbitrary order
    let mut table_a = HashMap::new();
    table_a.insert("id".to_string(), LuaValue::String("aura_z".to_string()));
    manager
        .displays
        .insert("aura_z".to_string(), LuaValue::Table(table_a));

    let mut table_b = HashMap::new();
    table_b.insert("id".to_string(), LuaValue::String("aura_a".to_string()));
    manager
        .displays
        .insert("aura_a".to_string(), LuaValue::Table(table_b));

    let output1 = manager.generate_lua();
    let output2 = manager.generate_lua();

    assert_eq!(
        output1, output2,
        "generate_lua() should produce identical output on repeated calls"
    );

    // Verify alphabetical ordering
    let pos_a = output1.find("[\"aura_a\"]").unwrap();
    let pos_z = output1.find("[\"aura_z\"]").unwrap();
    assert!(
        pos_a < pos_z,
        "aura_a should appear before aura_z in sorted output"
    );

    // Cleanup
    let _ = std::fs::remove_file(&temp_path);
}
