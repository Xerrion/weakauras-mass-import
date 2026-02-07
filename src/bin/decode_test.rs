//! Test binary to decode WeakAura strings and inspect structure

use std::fs;
use weakaura_mass_import::decoder::{LuaValue, WeakAuraDecoder};
use weakauras_codec::LuaValue as CodecLuaValue;

fn print_value(value: &LuaValue, indent: usize) {
    let prefix = "  ".repeat(indent);
    match value {
        LuaValue::Nil => println!("{}nil", prefix),
        LuaValue::Bool(b) => println!("{}{}", prefix, b),
        LuaValue::Number(n) => println!("{}{}", prefix, n),
        LuaValue::String(s) => {
            if s.len() > 100 {
                println!(
                    "{}\"{}...\" (truncated, {} chars)",
                    prefix,
                    &s[..100],
                    s.len()
                );
            } else {
                println!("{}\"{}\"", prefix, s);
            }
        }
        LuaValue::Array(arr) => {
            println!("{}Array[{}]:", prefix, arr.len());
            for (i, v) in arr.iter().enumerate() {
                println!("{}  [{}]:", prefix, i + 1);
                print_value(v, indent + 2);
            }
        }
        LuaValue::Table(table) => {
            println!("{}Table{{{}}}:", prefix, table.len());
            let mut keys: Vec<_> = table.keys().collect();
            keys.sort();
            for key in keys {
                println!("{}  [\"{}\"]:", prefix, key);
                print_value(&table[key], indent + 2);
            }
        }
        LuaValue::MixedTable { array, hash } => {
            println!(
                "{}MixedTable{{array: {}, hash: {}}}:",
                prefix,
                array.len(),
                hash.len()
            );
            println!("{}  -- Array part:", prefix);
            for (i, v) in array.iter().enumerate() {
                println!("{}    [{}]:", prefix, i + 1);
                print_value(v, indent + 3);
            }
            println!("{}  -- Hash part:", prefix);
            let mut keys: Vec<_> = hash.keys().collect();
            keys.sort();
            for key in keys {
                println!("{}    [\"{}\"]:", prefix, key);
                print_value(&hash[key], indent + 3);
            }
        }
    }
}

fn print_triggers_detail(value: &LuaValue) {
    // Find and print triggers structure in detail
    if let LuaValue::Table(table) = value {
        if let Some(triggers) = table.get("triggers") {
            println!("\n=== TRIGGERS STRUCTURE (DETAILED) ===");
            match triggers {
                LuaValue::Array(arr) => {
                    println!("Type: Array with {} elements", arr.len());
                    for (i, _v) in arr.iter().enumerate() {
                        println!("  [{}]: Table with trigger/untrigger", i + 1);
                    }
                }
                LuaValue::Table(t) => {
                    println!("Type: Table (pure hash) with {} keys:", t.len());
                    for key in t.keys() {
                        println!("  Key: \"{}\" (string)", key);
                    }
                }
                LuaValue::MixedTable { array, hash } => {
                    println!("Type: MixedTable");
                    println!(
                        "  Array part: {} elements (implicit indices 1..{})",
                        array.len(),
                        array.len()
                    );
                    println!("  Hash part: {} keys", hash.len());
                    for key in hash.keys() {
                        println!("    Key: \"{}\" (string)", key);
                    }
                }
                _ => println!("Unexpected type: {:?}", triggers),
            }
        }
    }
}

fn print_raw_triggers(value: &CodecLuaValue, path: &str) {
    if let CodecLuaValue::Map(map) = value {
        if path.is_empty() || path == "d" {
            // Look for triggers in the main aura data
            for (k, v) in map.iter() {
                let key_str = match k.as_value() {
                    CodecLuaValue::String(s) => s.clone(),
                    CodecLuaValue::Number(n) => format!("{}", n),
                    _ => continue,
                };

                if key_str == "d" {
                    // This is the aura data wrapper
                    print_raw_triggers(v, "d");
                } else if key_str == "triggers" {
                    println!("\n=== RAW TRIGGERS FROM CODEC ===");
                    print_raw_map(v, 0);
                }
            }
        }
    }
}

fn print_raw_map(value: &CodecLuaValue, indent: usize) {
    let prefix = "  ".repeat(indent);
    match value {
        CodecLuaValue::Map(map) => {
            println!("{}Map with {} entries:", prefix, map.len());
            for (k, v) in map.iter() {
                let key_desc = match k.as_value() {
                    CodecLuaValue::Number(n) => format!("Number({})", n),
                    CodecLuaValue::String(s) => format!("String(\"{}\")", s),
                    CodecLuaValue::Boolean(b) => format!("Bool({})", b),
                    _ => "Other".to_string(),
                };
                println!("{}  Key: {} =>", prefix, key_desc);
                print_raw_map(v, indent + 2);
            }
        }
        CodecLuaValue::Array(arr) => {
            println!("{}Array with {} elements", prefix, arr.len());
        }
        CodecLuaValue::String(s) => {
            if s.len() > 50 {
                println!("{}String(\"{}...\")", prefix, &s[..50]);
            } else {
                println!("{}String(\"{}\")", prefix, s);
            }
        }
        CodecLuaValue::Number(n) => {
            println!("{}Number({})", prefix, n);
        }
        CodecLuaValue::Boolean(b) => {
            println!("{}Bool({})", prefix, b);
        }
        CodecLuaValue::Null => {
            println!("{}Null", prefix);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: decode_test <file_or_string>");
        std::process::exit(1);
    }

    let input = if std::path::Path::new(&args[1]).exists() {
        fs::read_to_string(&args[1]).expect("Failed to read file")
    } else {
        args[1].clone()
    };

    let input = input.trim();

    // First, decode raw to see what the codec gives us
    println!("=== DECODING RAW FROM CODEC ===");
    if let Ok(Some(raw)) = weakauras_codec::decode(input.as_bytes(), Some(10 * 1024 * 1024)) {
        print_raw_triggers(&raw, "");
    }

    match WeakAuraDecoder::decode(input) {
        Ok(aura) => {
            println!("\n=== Decoded WeakAura ===");
            println!("ID: {}", aura.id);
            println!("UID: {:?}", aura.uid);
            println!("Region Type: {:?}", aura.region_type);
            println!("Is Group: {}", aura.is_group);
            println!("Children: {:?}", aura.children);
            println!("Encoding Version: {}", aura.encoding_version);
            println!("Child Data Count: {}", aura.child_data.len());

            // Print triggers detail
            print_triggers_detail(&aura.data);

            println!();
            println!("=== Main Aura Data Structure ===");
            print_value(&aura.data, 0);

            if !aura.child_data.is_empty() {
                println!();
                println!("=== Child Auras ({}) ===", aura.child_data.len());
                for (i, child) in aura.child_data.iter().enumerate() {
                    println!();
                    println!("--- Child {} ---", i + 1);
                    // Just print the key fields
                    if let LuaValue::Table(t) = child {
                        if let Some(LuaValue::String(id)) = t.get("id") {
                            println!("  id: {}", id);
                        }
                        if let Some(LuaValue::String(uid)) = t.get("uid") {
                            println!("  uid: {}", uid);
                        }
                        if let Some(LuaValue::String(rt)) = t.get("regionType") {
                            println!("  regionType: {}", rt);
                        }
                        if let Some(LuaValue::String(parent)) = t.get("parent") {
                            println!("  parent: {}", parent);
                        }
                        // Print triggers detail for child
                        print_triggers_detail(child);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to decode: {}", e);
            std::process::exit(1);
        }
    }
}
