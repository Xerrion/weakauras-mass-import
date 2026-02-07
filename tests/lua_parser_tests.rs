//! Tests for SavedVariables Lua parsing and serialization.

use std::collections::HashMap;

use weakaura_mass_import::decoder::LuaValue;
use weakaura_mass_import::lua_parser::LuaParser;

#[test]
fn test_parse_simple_table() {
    let result = LuaParser::parse("WeakAurasSaved = { displays = {} }").unwrap();
    assert!(result.displays.is_empty());
}

#[test]
fn test_serialize_mixed_table() {
    let mut hash = HashMap::new();
    hash.insert(
        "disjunctive".to_string(),
        LuaValue::String("all".to_string()),
    );
    hash.insert("activeTriggerMode".to_string(), LuaValue::Number(1.0));

    let mixed = LuaValue::MixedTable {
        array: vec![LuaValue::Table({
            let mut t = HashMap::new();
            t.insert("trigger".to_string(), LuaValue::Table(HashMap::new()));
            t
        })],
        hash,
    };

    let serialized = LuaParser::serialize(&mixed, 0);

    assert!(
        !serialized.contains("[\"1\"]"),
        "Mixed table should not have explicit [\"1\"] key"
    );
    assert!(
        serialized.contains("[\"activeTriggerMode\"]"),
        "Should have explicit activeTriggerMode key"
    );
    assert!(
        serialized.contains("[\"disjunctive\"]"),
        "Should have explicit disjunctive key"
    );
    assert!(
        serialized.contains("-- [1]"),
        "Should have comment showing implicit [1] index"
    );
}

#[test]
fn test_parse_mixed_table() {
    let input = r#"WeakAurasSaved = { displays = { ["test"] = {
        ["triggers"] = {
            { ["trigger"] = {} },
            ["disjunctive"] = "all",
        },
    } } }"#;

    let result = LuaParser::parse(input).unwrap();
    assert!(result.displays.contains_key("test"));

    if let Some(LuaValue::Table(test)) = result.displays.get("test") {
        if let Some(triggers) = test.get("triggers") {
            match triggers {
                LuaValue::MixedTable { array, hash } => {
                    assert_eq!(array.len(), 1, "Should have 1 array element");
                    assert!(
                        hash.contains_key("disjunctive"),
                        "Should have disjunctive key"
                    );
                }
                _ => panic!("triggers should be a MixedTable, got {:?}", triggers),
            }
        } else {
            panic!("test should have triggers key");
        }
    } else {
        panic!("displays should have test key as Table");
    }
}
