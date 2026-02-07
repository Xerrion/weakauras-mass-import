//! Tests for WeakAura string decoding and LuaValue types.

use std::collections::HashMap;

use weakaura_mass_import::decoder::{LuaValue, WeakAuraDecoder};
use weakaura_mass_import::lua_parser::LuaParser;

#[test]
fn test_detect_version() {
    assert_eq!(WeakAuraDecoder::detect_version("!WA:2!abc"), 2);
    assert_eq!(WeakAuraDecoder::detect_version("!abc"), 1);
    assert_eq!(WeakAuraDecoder::detect_version("abc"), 0);
}

#[test]
fn test_triggers_mixed_table_structure() {
    let mut hash = HashMap::new();
    hash.insert(
        "disjunctive".to_string(),
        LuaValue::String("all".to_string()),
    );
    hash.insert("activeTriggerMode".to_string(), LuaValue::Number(-10.0));

    let trigger1 = {
        let mut t = HashMap::new();
        t.insert(
            "trigger".to_string(),
            LuaValue::Table({
                let mut inner = HashMap::new();
                inner.insert("type".to_string(), LuaValue::String("aura2".to_string()));
                inner
            }),
        );
        LuaValue::Table(t)
    };

    let trigger2 = {
        let mut t = HashMap::new();
        t.insert(
            "trigger".to_string(),
            LuaValue::Table({
                let mut inner = HashMap::new();
                inner.insert("type".to_string(), LuaValue::String("spell".to_string()));
                inner
            }),
        );
        LuaValue::Table(t)
    };

    let mixed = LuaValue::MixedTable {
        array: vec![trigger1, trigger2],
        hash,
    };

    if let LuaValue::MixedTable { array, hash } = &mixed {
        assert_eq!(array.len(), 2, "Should have 2 triggers in array part");
        assert_eq!(hash.len(), 2, "Should have 2 entries in hash part");
        assert!(hash.contains_key("disjunctive"));
        assert!(hash.contains_key("activeTriggerMode"));
    } else {
        panic!("Expected MixedTable");
    }
}

#[test]
fn test_triggers_serialization_no_string_numeric_keys() {
    let mut hash = HashMap::new();
    hash.insert(
        "disjunctive".to_string(),
        LuaValue::String("any".to_string()),
    );
    hash.insert("activeTriggerMode".to_string(), LuaValue::Number(-10.0));

    let trigger1 = {
        let mut t = HashMap::new();
        t.insert(
            "trigger".to_string(),
            LuaValue::Table({
                let mut inner = HashMap::new();
                inner.insert("type".to_string(), LuaValue::String("aura2".to_string()));
                inner
            }),
        );
        t.insert("untrigger".to_string(), LuaValue::Table(HashMap::new()));
        LuaValue::Table(t)
    };

    let trigger2 = {
        let mut t = HashMap::new();
        t.insert(
            "trigger".to_string(),
            LuaValue::Table({
                let mut inner = HashMap::new();
                inner.insert("type".to_string(), LuaValue::String("spell".to_string()));
                inner.insert("spellName".to_string(), LuaValue::Number(12345.0));
                inner
            }),
        );
        t.insert("untrigger".to_string(), LuaValue::Table(HashMap::new()));
        LuaValue::Table(t)
    };

    let triggers = LuaValue::MixedTable {
        array: vec![trigger1, trigger2],
        hash,
    };

    let serialized = LuaParser::serialize(&triggers, 0);

    assert!(
        !serialized.contains("[\"1\"]"),
        "Serialized triggers should NOT contain string key [\"1\"]. Got:\n{}",
        serialized
    );
    assert!(
        !serialized.contains("[\"2\"]"),
        "Serialized triggers should NOT contain string key [\"2\"]. Got:\n{}",
        serialized
    );
    assert!(
        serialized.contains("[\"disjunctive\"]"),
        "Should have explicit disjunctive key"
    );
    assert!(
        serialized.contains("[\"activeTriggerMode\"]"),
        "Should have explicit activeTriggerMode key"
    );
    assert!(
        serialized.contains("-- [1]"),
        "Should have comment showing implicit [1] index"
    );
    assert!(
        serialized.contains("-- [2]"),
        "Should have comment showing implicit [2] index"
    );
}

#[test]
fn test_pure_array_serialization() {
    let arr = LuaValue::Array(vec![
        LuaValue::String("first".to_string()),
        LuaValue::String("second".to_string()),
        LuaValue::String("third".to_string()),
    ]);

    let serialized = LuaParser::serialize(&arr, 0);

    assert!(
        !serialized.contains("[\"1\"]"),
        "Array should not have string key [\"1\"]"
    );
    assert!(
        !serialized.contains("[\"2\"]"),
        "Array should not have string key [\"2\"]"
    );
    assert!(
        !serialized.contains("[\"3\"]"),
        "Array should not have string key [\"3\"]"
    );
}

#[test]
fn test_convert_lua_value_preserves_numeric_keys() {
    let mut hash = HashMap::new();
    hash.insert(
        "disjunctive".to_string(),
        LuaValue::String("any".to_string()),
    );

    let trigger1 = LuaValue::Table({
        let mut t = HashMap::new();
        t.insert("trigger".to_string(), LuaValue::Table(HashMap::new()));
        t
    });

    let trigger2 = LuaValue::Table({
        let mut t = HashMap::new();
        t.insert("trigger".to_string(), LuaValue::Table(HashMap::new()));
        t
    });

    let mixed = LuaValue::MixedTable {
        array: vec![trigger1, trigger2],
        hash,
    };

    let serialized = LuaParser::serialize(&mixed, 0);

    assert!(
        !serialized.contains("[\"1\"]"),
        "Should not have string key [\"1\"]"
    );
    assert!(
        !serialized.contains("[\"2\"]"),
        "Should not have string key [\"2\"]"
    );
    assert!(
        serialized.contains("-- [1]"),
        "Should show implicit index 1"
    );
    assert!(
        serialized.contains("-- [2]"),
        "Should show implicit index 2"
    );
}
