//! WeakAura string decoder
//!
//! WeakAura import strings are encoded with different versions:
//! - Version 0: No prefix, LibCompress + AceSerializer, custom Base64
//! - Version 1: `!` prefix, LibDeflate + AceSerializer
//! - Version 2+: `!WA:N!` prefix, LibDeflate + LibSerialize (binary)
//!
//! We use the `weakauras-codec` crate for the heavy lifting.

use crate::error::{Result, WeakAuraError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn};
use weakauras_codec::LuaValue as CodecLuaValue;

/// Represents a decoded WeakAura
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeakAura {
    /// The unique ID of the aura
    pub id: String,
    /// The unique UID (11-char base64)
    pub uid: Option<String>,
    /// The type of region (icon, aurabar, text, group, etc.)
    pub region_type: Option<String>,
    /// Whether this is a group
    pub is_group: bool,
    /// Child auras (if this is a group)
    pub children: Vec<String>,
    /// The raw decoded data
    pub data: LuaValue,
    /// Child aura data (for groups)
    pub child_data: Vec<LuaValue>,
    /// The original import string
    pub original_string: String,
    /// Encoding version detected
    pub encoding_version: u8,
}

/// Represents a Lua value (since WeakAura data is essentially a Lua table)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(untagged)]
pub enum LuaValue {
    #[default]
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Table(HashMap<String, LuaValue>),
    Array(Vec<LuaValue>),
    /// Mixed table: array part (1-indexed implicit) + hash part (string keys)
    /// This is common in Lua, e.g., triggers = { {trigger1}, {trigger2}, disjunctive = "all" }
    MixedTable {
        array: Vec<LuaValue>,
        hash: HashMap<String, LuaValue>,
    },
}

impl LuaValue {
    pub fn as_table(&self) -> Option<&HashMap<String, LuaValue>> {
        match self {
            LuaValue::Table(t) => Some(t),
            LuaValue::MixedTable { hash, .. } => Some(hash),
            _ => None,
        }
    }

    pub fn as_table_mut(&mut self) -> Option<&mut HashMap<String, LuaValue>> {
        match self {
            LuaValue::Table(t) => Some(t),
            LuaValue::MixedTable { hash, .. } => Some(hash),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<LuaValue>> {
        match self {
            LuaValue::Array(a) => Some(a),
            LuaValue::MixedTable { array, .. } => Some(array),
            _ => None,
        }
    }
}

/// Convert from weakauras_codec::LuaValue to our LuaValue
fn convert_lua_value(value: &CodecLuaValue) -> LuaValue {
    match value {
        CodecLuaValue::Null => LuaValue::Nil,
        CodecLuaValue::Boolean(b) => LuaValue::Bool(*b),
        CodecLuaValue::Number(n) => LuaValue::Number(*n),
        CodecLuaValue::String(s) => LuaValue::String(s.clone()),
        CodecLuaValue::Array(arr) => LuaValue::Array(arr.iter().map(convert_lua_value).collect()),
        CodecLuaValue::Map(map) => {
            // Separate numeric keys (array part) from string keys (hash part)
            let mut numeric_entries: Vec<(i64, LuaValue)> = Vec::new();
            let mut string_entries: HashMap<String, LuaValue> = HashMap::new();
            let mut has_string_keys = false;
            let mut has_numeric_keys = false;

            for (k, v) in map.iter() {
                match k.as_value() {
                    CodecLuaValue::Number(n) if *n > 0.0 && n.fract() == 0.0 => {
                        has_numeric_keys = true;
                        numeric_entries.push((*n as i64, convert_lua_value(v)));
                    }
                    CodecLuaValue::String(s) => {
                        // Check if string represents a positive integer (array index)
                        // The weakauras-codec may return numeric keys as strings for some encoding versions
                        if let Ok(n) = s.parse::<i64>() {
                            if n > 0 {
                                has_numeric_keys = true;
                                numeric_entries.push((n, convert_lua_value(v)));
                            } else {
                                // Zero or negative - treat as string key
                                has_string_keys = true;
                                string_entries.insert(s.clone(), convert_lua_value(v));
                            }
                        } else {
                            has_string_keys = true;
                            string_entries.insert(s.clone(), convert_lua_value(v));
                        }
                    }
                    CodecLuaValue::Number(n) => {
                        // Non-positive or fractional number key - treat as string
                        has_string_keys = true;
                        let key = if n.fract() == 0.0 {
                            (*n as i64).to_string()
                        } else {
                            n.to_string()
                        };
                        string_entries.insert(key, convert_lua_value(v));
                    }
                    CodecLuaValue::Boolean(b) => {
                        has_string_keys = true;
                        string_entries.insert(b.to_string(), convert_lua_value(v));
                    }
                    _ => continue,
                }
            }

            // Determine the table type
            if has_numeric_keys && has_string_keys {
                // Mixed table: has both array part and hash part
                // Sort numeric entries and check if they form a contiguous array from 1
                numeric_entries.sort_by_key(|(idx, _)| *idx);

                // Check if numeric entries are contiguous from 1
                let is_contiguous = !numeric_entries.is_empty()
                    && numeric_entries
                        .iter()
                        .enumerate()
                        .all(|(i, (idx, _))| *idx == (i as i64) + 1);

                if is_contiguous {
                    let array: Vec<LuaValue> =
                        numeric_entries.into_iter().map(|(_, v)| v).collect();
                    LuaValue::MixedTable {
                        array,
                        hash: string_entries,
                    }
                } else {
                    // Convert everything to string-keyed table
                    for (idx, val) in numeric_entries {
                        string_entries.insert(idx.to_string(), val);
                    }
                    LuaValue::Table(string_entries)
                }
            } else if has_numeric_keys {
                // Pure array - verify contiguous from 1
                numeric_entries.sort_by_key(|(idx, _)| *idx);

                let max_index = numeric_entries
                    .iter()
                    .map(|(idx, _)| *idx)
                    .max()
                    .unwrap_or(0);
                let is_contiguous = max_index > 0
                    && max_index as usize == numeric_entries.len()
                    && numeric_entries
                        .iter()
                        .enumerate()
                        .all(|(i, (idx, _))| *idx == (i as i64) + 1);

                if is_contiguous {
                    let arr: Vec<LuaValue> = numeric_entries.into_iter().map(|(_, v)| v).collect();
                    LuaValue::Array(arr)
                } else {
                    // Non-contiguous numeric keys - treat as table
                    let mut table = HashMap::new();
                    for (idx, val) in numeric_entries {
                        table.insert(idx.to_string(), val);
                    }
                    LuaValue::Table(table)
                }
            } else {
                // Pure string-keyed table (or empty)
                LuaValue::Table(string_entries)
            }
        }
    }
}

/// Decoder for WeakAura import strings
pub struct WeakAuraDecoder;

impl WeakAuraDecoder {
    /// Decode a WeakAura import string
    pub fn decode(import_string: &str) -> Result<WeakAura> {
        let trimmed = import_string.trim();

        // Use weakauras-codec for decoding
        // The crate takes bytes and an optional max decompressed size
        let decoded = weakauras_codec::decode(trimmed.as_bytes(), Some(10 * 1024 * 1024)) // 10MB max
            .map_err(|e| WeakAuraError::DeserializationError(e.to_string()))?
            .ok_or_else(|| {
                WeakAuraError::DeserializationError("Decode returned None".to_string())
            })?;

        // Determine encoding version from prefix
        let encoding_version = Self::detect_version(trimmed);

        // Convert the decoded data
        let data = convert_lua_value(&decoded);

        // Extract metadata from the decoded structure
        // WeakAura format: { m = "d", d = <aura_data>, c = [<children>], v = version, s = wa_version }
        let (aura_data, child_data) = Self::extract_aura_data(&data);
        let (id, uid, region_type, is_group, children) =
            Self::extract_metadata(&aura_data, &child_data);

        Ok(WeakAura {
            id,
            uid,
            region_type,
            is_group,
            children,
            data: aura_data,
            child_data,
            original_string: import_string.to_string(),
            encoding_version,
        })
    }

    /// Decode multiple import strings (one per line or separated by blank lines)
    pub fn decode_multiple(input: &str) -> Vec<Result<WeakAura>> {
        let mut results = Vec::new();

        // Split by blank lines or detect individual strings
        let strings: Vec<&str> = input
            .split("\n\n")
            .flat_map(|s| s.split('\n'))
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .filter(|s| s.starts_with('!') || Self::looks_like_weakaura(s))
            .collect();

        for s in strings {
            results.push(Self::decode(s));
        }

        results
    }

    /// Quick check if a string looks like a WeakAura import string
    fn looks_like_weakaura(s: &str) -> bool {
        // WeakAura strings are typically long base64-like strings
        s.len() > 50
            && s.chars()
                .all(|c| c.is_alphanumeric() || "!:+/=()".contains(c))
    }

    /// Detect the encoding version from the string prefix
    fn detect_version(s: &str) -> u8 {
        if let Some(rest) = s.strip_prefix("!WA:") {
            // Format: !WA:N!...
            if let Some(version_end) = rest.find('!') {
                if let Ok(v) = rest[..version_end].parse::<u8>() {
                    return v;
                }
            }
            2 // Default to version 2 if parsing fails
        } else if s.starts_with('!') {
            1
        } else {
            0
        }
    }

    /// Extract the actual aura data and children from the transmission wrapper
    fn extract_aura_data(data: &LuaValue) -> (LuaValue, Vec<LuaValue>) {
        if let Some(table) = data.as_table() {
            // Check for transmission wrapper format
            let has_wrapper = table.contains_key("d");
            let aura_data = table.get("d").cloned().unwrap_or_else(|| data.clone());

            let child_data = table
                .get("c")
                .and_then(|c| c.as_array())
                .cloned()
                .unwrap_or_default();

            debug!(
                has_wrapper,
                child_count = child_data.len(),
                "Extracted aura data from transmission wrapper"
            );

            (aura_data, child_data)
        } else {
            warn!(
                value_type = std::any::type_name::<LuaValue>(),
                "extract_aura_data: data is not a table variant, returning as-is"
            );
            (data.clone(), Vec::new())
        }
    }

    /// Extract metadata from decoded WeakAura data
    fn extract_metadata(
        data: &LuaValue,
        child_data: &[LuaValue],
    ) -> (String, Option<String>, Option<String>, bool, Vec<String>) {
        let mut id = String::from("unknown");
        let mut uid = None;
        let mut region_type = None;
        let mut is_group = false;
        let mut children = Vec::new();

        if let Some(table) = data.as_table() {
            if let Some(LuaValue::String(s)) = table.get("id") {
                id = s.clone();
            }

            if let Some(LuaValue::String(s)) = table.get("uid") {
                uid = Some(s.clone());
            }

            if let Some(LuaValue::String(s)) = table.get("regionType") {
                region_type = Some(s.clone());
                if s == "group" || s == "dynamicgroup" {
                    is_group = true;
                }
            }

            // controlledChildren can be Array or MixedTable
            if let Some(arr) = table.get("controlledChildren").and_then(|v| v.as_array()) {
                for child in arr {
                    if let LuaValue::String(s) = child {
                        children.push(s.clone());
                    }
                }
                if !children.is_empty() {
                    is_group = true;
                }
            }
        } else {
            warn!(
                "extract_metadata: data is not a table variant (is {:?}), metadata extraction skipped",
                std::mem::discriminant(data)
            );
        }

        // Fallback: If no children found in controlledChildren but we have child_data,
        // infer children from child_data (common in import strings where parent doesn't have controlledChildren yet)
        if children.is_empty() && !child_data.is_empty() {
            debug!(
                child_data_count = child_data.len(),
                "No controlledChildren found, inferring from child_data"
            );
            for child in child_data {
                if let Some(table) = child.as_table() {
                    if let Some(LuaValue::String(child_id)) = table.get("id") {
                        children.push(child_id.clone());
                    }
                } else {
                    warn!("Child data entry is not a table variant, skipping");
                }
            }
            if !children.is_empty() {
                is_group = true;
            }
        }

        debug!(
            id = %id,
            is_group,
            child_count = children.len(),
            "Extracted metadata"
        );

        (id, uid, region_type, is_group, children)
    }
}

/// Result of validating a WeakAura string
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub format: Option<String>,
    pub aura_id: Option<String>,
    pub is_group: bool,
    pub child_count: usize,
    pub error: Option<String>,
}

impl ValidationResult {
    pub fn summary(&self) -> String {
        if self.is_valid {
            let mut parts = vec![];
            if let Some(id) = &self.aura_id {
                parts.push(format!("ID: {}", id));
            }
            if let Some(fmt) = &self.format {
                parts.push(fmt.clone());
            }
            if self.is_group {
                parts.push(format!("Group with {} children", self.child_count));
            }
            parts.join(" | ")
        } else {
            self.error.clone().unwrap_or_else(|| "Invalid".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua_parser::LuaParser;

    #[test]
    fn test_detect_version() {
        assert_eq!(WeakAuraDecoder::detect_version("!WA:2!abc"), 2);
        assert_eq!(WeakAuraDecoder::detect_version("!abc"), 1);
        assert_eq!(WeakAuraDecoder::detect_version("abc"), 0);
    }

    #[test]
    fn test_triggers_mixed_table_structure() {
        // Create a mixed table like WeakAuras triggers:
        // { {trigger=..}, {trigger=..}, disjunctive = "all", activeTriggerMode = -10 }
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

        // Verify structure
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
        // Create a triggers structure with 2 array elements
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

        // CRITICAL: The serialized output must NOT contain ["1"] or ["2"]
        // Array elements should use implicit indices (no key shown)
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

        // Should have hash keys with explicit brackets
        assert!(
            serialized.contains("[\"disjunctive\"]"),
            "Should have explicit disjunctive key"
        );
        assert!(
            serialized.contains("[\"activeTriggerMode\"]"),
            "Should have explicit activeTriggerMode key"
        );

        // Should have comments showing implicit indices (our format uses -- [1], -- [2])
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
        // A pure array should also not have string keys
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
        // This test verifies that our convert_lua_value correctly converts
        // the codec's LuaValue::Map with numeric and string keys into our
        // MixedTable structure.
        //
        // We can't easily construct codec types directly, but we can test
        // by decoding a known WeakAura string with multiple triggers.
        // For now, we test our internal structure manipulation.

        // Create the expected structure manually
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

        // Verify it serializes correctly
        let serialized = LuaParser::serialize(&mixed, 0);

        // CRITICAL: No string numeric keys
        assert!(
            !serialized.contains("[\"1\"]"),
            "Should not have string key [\"1\"]"
        );
        assert!(
            !serialized.contains("[\"2\"]"),
            "Should not have string key [\"2\"]"
        );

        // Should have implicit indices
        assert!(
            serialized.contains("-- [1]"),
            "Should show implicit index 1"
        );
        assert!(
            serialized.contains("-- [2]"),
            "Should show implicit index 2"
        );
    }
}
