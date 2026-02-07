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

#[test]
fn test_parse_boolean_array_elements() {
    // This was the exact pattern causing "Expected '='" error:
    // boolean values as implicit array elements in a table
    let input = r#"WeakAurasSaved = { displays = { ["test"] = {
        ["default"] = {
            true, -- [1]
            false, -- [2]
            false, -- [3]
            true, -- [4]
        },
    } } }"#;

    let result = LuaParser::parse(input).unwrap();
    let test = result.displays.get("test").unwrap();
    if let LuaValue::Table(test_table) = test {
        if let Some(LuaValue::Array(arr)) = test_table.get("default") {
            assert_eq!(arr.len(), 4);
            assert_eq!(arr[0], LuaValue::Bool(true));
            assert_eq!(arr[1], LuaValue::Bool(false));
            assert_eq!(arr[2], LuaValue::Bool(false));
            assert_eq!(arr[3], LuaValue::Bool(true));
        } else {
            panic!("default should be an Array");
        }
    } else {
        panic!("test should be a Table");
    }
}

#[test]
fn test_parse_nil_array_elements() {
    let input = r#"WeakAurasSaved = { displays = { ["test"] = {
        ["data"] = {
            nil, -- [1]
            "hello", -- [2]
            nil, -- [3]
        },
    } } }"#;

    let result = LuaParser::parse(input).unwrap();
    let test = result.displays.get("test").unwrap();
    if let LuaValue::Table(test_table) = test {
        if let Some(LuaValue::Array(arr)) = test_table.get("data") {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], LuaValue::Nil);
            assert_eq!(arr[1], LuaValue::String("hello".to_string()));
            assert_eq!(arr[2], LuaValue::Nil);
        } else {
            panic!("data should be an Array");
        }
    } else {
        panic!("test should be a Table");
    }
}

#[test]
fn test_parse_math_huge() {
    let input = r#"WeakAurasSaved = { displays = { ["test"] = {
        ["pos_inf"] = math.huge,
        ["neg_inf"] = -math.huge,
    } } }"#;

    let result = LuaParser::parse(input).unwrap();
    let test = result.displays.get("test").unwrap();
    if let LuaValue::Table(test_table) = test {
        if let LuaValue::Number(n) = test_table.get("pos_inf").unwrap() {
            assert!(
                n.is_infinite() && n.is_sign_positive(),
                "Expected +inf, got {}",
                n
            );
        } else {
            panic!("pos_inf should be a Number");
        }
        if let LuaValue::Number(n) = test_table.get("neg_inf").unwrap() {
            assert!(
                n.is_infinite() && n.is_sign_negative(),
                "Expected -inf, got {}",
                n
            );
        } else {
            panic!("neg_inf should be a Number");
        }
    } else {
        panic!("test should be a Table");
    }
}

#[test]
fn test_parse_nan() {
    let input = r#"WeakAurasSaved = { displays = { ["test"] = {
        ["nan_val"] = (0/0),
    } } }"#;

    let result = LuaParser::parse(input).unwrap();
    let test = result.displays.get("test").unwrap();
    if let LuaValue::Table(test_table) = test {
        if let LuaValue::Number(n) = test_table.get("nan_val").unwrap() {
            assert!(n.is_nan(), "Expected NaN, got {}", n);
        } else {
            panic!("nan_val should be a Number");
        }
    } else {
        panic!("test should be a Table");
    }
}

#[test]
fn test_roundtrip_boolean_array() {
    // Serialize an array with booleans, then parse it back
    let original = LuaValue::Array(vec![
        LuaValue::Bool(true),
        LuaValue::Bool(false),
        LuaValue::Bool(true),
    ]);

    let serialized = LuaParser::serialize(&original, 0);
    let wrapped = format!(
        "WeakAurasSaved = {{ displays = {{ [\"test\"] = {{ [\"arr\"] = {} }} }} }}",
        serialized
    );

    let result = LuaParser::parse(&wrapped).unwrap();
    let test = result.displays.get("test").unwrap();
    if let LuaValue::Table(test_table) = test {
        if let Some(LuaValue::Array(arr)) = test_table.get("arr") {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], LuaValue::Bool(true));
            assert_eq!(arr[1], LuaValue::Bool(false));
            assert_eq!(arr[2], LuaValue::Bool(true));
        } else {
            panic!("arr should be an Array, got {:?}", test_table.get("arr"));
        }
    } else {
        panic!("test should be a Table");
    }
}

#[test]
fn test_roundtrip_special_numbers() {
    // Serialize infinity and NaN, then parse them back
    let mut table = HashMap::new();
    table.insert("inf".to_string(), LuaValue::Number(f64::INFINITY));
    table.insert("neg_inf".to_string(), LuaValue::Number(f64::NEG_INFINITY));
    table.insert("nan".to_string(), LuaValue::Number(f64::NAN));
    let original = LuaValue::Table(table);

    let serialized = LuaParser::serialize(&original, 0);
    assert!(
        serialized.contains("math.huge"),
        "Should serialize +inf as math.huge"
    );
    assert!(
        serialized.contains("-math.huge"),
        "Should serialize -inf as -math.huge"
    );
    assert!(
        serialized.contains("(0/0)"),
        "Should serialize NaN as (0/0)"
    );

    let wrapped = format!(
        "WeakAurasSaved = {{ displays = {{ [\"test\"] = {} }} }}",
        serialized
    );

    let result = LuaParser::parse(&wrapped).unwrap();
    let test = result.displays.get("test").unwrap();
    if let LuaValue::Table(test_table) = test {
        if let LuaValue::Number(n) = test_table.get("inf").unwrap() {
            assert!(n.is_infinite() && n.is_sign_positive());
        } else {
            panic!("inf should be a Number");
        }
        if let LuaValue::Number(n) = test_table.get("neg_inf").unwrap() {
            assert!(n.is_infinite() && n.is_sign_negative());
        } else {
            panic!("neg_inf should be a Number");
        }
        if let LuaValue::Number(n) = test_table.get("nan").unwrap() {
            assert!(n.is_nan());
        } else {
            panic!("nan should be a Number");
        }
    } else {
        panic!("test should be a Table");
    }
}

#[test]
fn test_parse_mixed_table_with_booleans() {
    // Mixed table: booleans in array part + string keys in hash part
    let input = r#"WeakAurasSaved = { displays = { ["test"] = {
        ["triggers"] = {
            true, -- [1]
            false, -- [2]
            ["disjunctive"] = "all",
        },
    } } }"#;

    let result = LuaParser::parse(input).unwrap();
    let test = result.displays.get("test").unwrap();
    if let LuaValue::Table(test_table) = test {
        match test_table.get("triggers").unwrap() {
            LuaValue::MixedTable { array, hash } => {
                assert_eq!(array.len(), 2);
                assert_eq!(array[0], LuaValue::Bool(true));
                assert_eq!(array[1], LuaValue::Bool(false));
                assert_eq!(
                    hash.get("disjunctive"),
                    Some(&LuaValue::String("all".to_string()))
                );
            }
            other => panic!("triggers should be MixedTable, got {:?}", other),
        }
    } else {
        panic!("test should be a Table");
    }
}
