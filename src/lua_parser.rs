//! Lua parser for WoW SavedVariables files
//!
//! SavedVariables files are Lua scripts that assign values to global variables.
//! For WeakAuras, the main variables are:
//! - `WeakAurasSaved` - Contains all saved auras and settings
//! - `WeakAurasDisplays` - Contains display metadata

use crate::decoder::LuaValue;
use crate::error::{Result, WeakAuraError};
use crate::util;
use std::collections::HashMap;

/// Represents the parsed WeakAuras SavedVariables
#[derive(Debug, Clone, Default)]
pub struct WeakAurasSaved {
    /// All saved displays/auras
    pub displays: HashMap<String, LuaValue>,
    /// Other data
    pub other: HashMap<String, LuaValue>,
}

/// Parser for Lua SavedVariables files
pub struct LuaParser;

impl LuaParser {
    /// Parse SavedVariables content
    pub fn parse(content: &str) -> Result<WeakAurasSaved> {
        let mut saved = WeakAurasSaved::default();

        // Find WeakAurasSaved assignment
        // Format: WeakAurasSaved = { ... }
        if let Some(start) = content.find("WeakAurasSaved") {
            if let Some(eq_pos) = content[start..].find('=') {
                let table_start = start + eq_pos + 1;
                if let Some(brace_pos) = content[table_start..].find('{') {
                    let table_content_start = table_start + brace_pos;
                    if let Some((table_value, _)) =
                        Self::parse_table(&content[table_content_start..])?
                    {
                        if let Some(table) = table_value.as_table() {
                            // Extract displays
                            if let Some(displays) = table.get("displays").and_then(|v| v.as_table())
                            {
                                saved.displays = displays.clone();
                            }
                            // Store other fields
                            for (key, value) in table {
                                if key != "displays" {
                                    saved.other.insert(key.clone(), value.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(saved)
    }

    /// Parse a Lua table from string
    fn parse_table(input: &str) -> Result<Option<(LuaValue, usize)>> {
        let input = input.trim();
        if !input.starts_with('{') {
            return Ok(None);
        }

        let mut parser = LuaTableParser::new(input);
        let value = parser.parse_table()?;
        Ok(Some((value, parser.pos)))
    }

    /// Serialize a LuaValue back to Lua string format
    pub fn serialize(value: &LuaValue, indent: usize) -> String {
        let indent_str = "\t".repeat(indent);
        match value {
            LuaValue::Nil => "nil".to_string(),
            LuaValue::Bool(b) => if *b { "true" } else { "false" }.to_string(),
            LuaValue::Number(n) => {
                if n.is_nan() {
                    "(0/0)".to_string()
                } else if n.is_infinite() {
                    if n.is_sign_positive() {
                        "math.huge".to_string()
                    } else {
                        "-math.huge".to_string()
                    }
                } else if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            LuaValue::String(s) => format!("\"{}\"", util::escape_lua_string(s)),
            LuaValue::Array(arr) => {
                let mut result = String::from("{\n");
                for (i, v) in arr.iter().enumerate() {
                    result.push_str(&format!(
                        "{}\t{}, -- [{}]\n",
                        indent_str,
                        Self::serialize(v, indent + 1),
                        i + 1
                    ));
                }
                result.push_str(&format!("{}}}", indent_str));
                result
            }
            LuaValue::Table(table) => {
                let mut result = String::from("{\n");
                let mut keys: Vec<_> = table.keys().collect();
                keys.sort();
                for key in keys {
                    let value = &table[key];
                    // All keys in WeakAuras SavedVariables use ["key"] format
                    let key_str = format!("[\"{}\"]", util::escape_lua_string(key));
                    result.push_str(&format!(
                        "{}\t{} = {},\n",
                        indent_str,
                        key_str,
                        Self::serialize(value, indent + 1)
                    ));
                }
                result.push_str(&format!("{}}}", indent_str));
                result
            }
            LuaValue::MixedTable { array, hash } => {
                // Mixed table: array part first (implicit indices), then hash part (explicit keys)
                let mut result = String::from("{\n");

                // Array part - use implicit indices (no key shown)
                for (i, v) in array.iter().enumerate() {
                    result.push_str(&format!(
                        "{}\t{}, -- [{}]\n",
                        indent_str,
                        Self::serialize(v, indent + 1),
                        i + 1
                    ));
                }

                // Hash part - use explicit string keys
                let mut keys: Vec<_> = hash.keys().collect();
                keys.sort();
                for key in keys {
                    let value = &hash[key];
                    let key_str = format!("[\"{}\"]", util::escape_lua_string(key));
                    result.push_str(&format!(
                        "{}\t{} = {},\n",
                        indent_str,
                        key_str,
                        Self::serialize(value, indent + 1)
                    ));
                }

                result.push_str(&format!("{}}}", indent_str));
                result
            }
        }
    }
}

/// Internal parser for Lua tables
struct LuaTableParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> LuaTableParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn parse_table(&mut self) -> Result<LuaValue> {
        self.skip_whitespace();

        if !self.consume('{') {
            return Err(WeakAuraError::LuaParseError("Expected '{'".to_string()));
        }

        // Track implicit array indices (elements without explicit keys)
        let mut implicit_array: Vec<LuaValue> = Vec::new();
        // Track explicit string keys
        let mut explicit_hash: HashMap<String, LuaValue> = HashMap::new();
        // Track explicit numeric keys (might be sparse or out of order)
        let mut explicit_numeric: Vec<(usize, LuaValue)> = Vec::new();

        loop {
            self.skip_whitespace();
            self.skip_comments();
            self.skip_whitespace();

            if self.peek() == Some('}') {
                self.consume('}');
                break;
            }

            // Parse key-value pair
            let (key, value) = self.parse_key_value()?;

            match key {
                None => {
                    // Implicit index - this is array-like
                    implicit_array.push(value);
                }
                Some(k) => {
                    // Check if this is a numeric key
                    if let Ok(n) = k.parse::<usize>() {
                        if n > 0 {
                            explicit_numeric.push((n, value));
                        } else {
                            explicit_hash.insert(k, value);
                        }
                    } else {
                        // String key
                        explicit_hash.insert(k, value);
                    }
                }
            }

            self.skip_whitespace();
            self.consume(','); // Optional comma
        }

        // Determine the final structure
        let has_implicit = !implicit_array.is_empty();
        let has_explicit_numeric = !explicit_numeric.is_empty();
        let has_hash = !explicit_hash.is_empty();

        if has_implicit && !has_explicit_numeric && !has_hash {
            // Pure array with implicit indices only
            Ok(LuaValue::Array(implicit_array))
        } else if !has_implicit && has_explicit_numeric && !has_hash {
            // Only explicit numeric keys - check if it forms a contiguous array
            explicit_numeric.sort_by_key(|(idx, _)| *idx);
            let max_index = explicit_numeric
                .iter()
                .map(|(idx, _)| *idx)
                .max()
                .unwrap_or(0);
            let is_contiguous = max_index == explicit_numeric.len()
                && explicit_numeric
                    .iter()
                    .enumerate()
                    .all(|(i, (idx, _))| *idx == i + 1);

            if is_contiguous {
                let arr: Vec<LuaValue> = explicit_numeric.into_iter().map(|(_, v)| v).collect();
                Ok(LuaValue::Array(arr))
            } else {
                // Sparse numeric keys - convert to table
                let mut table = HashMap::new();
                for (idx, val) in explicit_numeric {
                    table.insert(idx.to_string(), val);
                }
                Ok(LuaValue::Table(table))
            }
        } else if (has_implicit || has_explicit_numeric) && has_hash {
            // Mixed table: combine array part + hash part
            // Merge implicit array and explicit numeric into array part
            let mut array = implicit_array;

            // Handle explicit numeric keys that extend or overwrite the array
            if has_explicit_numeric {
                explicit_numeric.sort_by_key(|(idx, _)| *idx);
                for (idx, val) in explicit_numeric {
                    if idx == 0 {
                        // Zero index - not valid for Lua arrays, put in hash
                        explicit_hash.insert(idx.to_string(), val);
                    } else if idx <= array.len() {
                        // Index within existing array bounds - replace/overwrite
                        array[idx - 1] = val;
                    } else if idx == array.len() + 1 {
                        // Index is exactly next position - append
                        array.push(val);
                    } else {
                        // Index beyond current array - fill gaps with Nil and add
                        while array.len() < idx - 1 {
                            array.push(LuaValue::Nil);
                        }
                        array.push(val);
                    }
                }
            }

            if array.is_empty() {
                Ok(LuaValue::Table(explicit_hash))
            } else {
                Ok(LuaValue::MixedTable {
                    array,
                    hash: explicit_hash,
                })
            }
        } else if has_hash {
            // Only string keys
            Ok(LuaValue::Table(explicit_hash))
        } else {
            // Empty table
            Ok(LuaValue::Table(HashMap::new()))
        }
    }

    fn parse_key_value(&mut self) -> Result<(Option<String>, LuaValue)> {
        self.skip_whitespace();

        // Check for explicit key
        let key = if self.peek() == Some('[') {
            // [key] = value or ["key"] = value
            self.consume('[');
            self.skip_whitespace();
            let key = self.parse_value()?;
            self.skip_whitespace();
            self.consume(']');
            self.skip_whitespace();
            self.consume('=');
            Some(match key {
                LuaValue::String(s) => s,
                LuaValue::Number(n) => n.to_string(),
                _ => return Err(WeakAuraError::LuaParseError("Invalid key type".to_string())),
            })
        } else if self.peek_identifier() {
            // identifier = value
            let ident = self.parse_identifier()?;
            self.skip_whitespace();
            if self.peek() == Some('=') {
                self.consume('=');
                Some(ident)
            } else {
                // Not a key, backtrack (this is tricky)
                return Err(WeakAuraError::LuaParseError("Expected '='".to_string()));
            }
        } else {
            // Implicit index
            None
        };

        self.skip_whitespace();
        let value = self.parse_value()?;

        Ok((key, value))
    }

    fn parse_value(&mut self) -> Result<LuaValue> {
        self.skip_whitespace();

        match self.peek() {
            Some('{') => self.parse_table(),
            Some('"') | Some('\'') => self.parse_string(),
            Some('[') if self.peek_long_string() => self.parse_long_string(),
            Some(c) if c.is_ascii_digit() || c == '-' || c == '.' => self.parse_number(),
            Some('t') if self.peek_word("true") => {
                self.advance_by(4);
                Ok(LuaValue::Bool(true))
            }
            Some('f') if self.peek_word("false") => {
                self.advance_by(5);
                Ok(LuaValue::Bool(false))
            }
            Some('n') if self.peek_word("nil") => {
                self.advance_by(3);
                Ok(LuaValue::Nil)
            }
            _ => {
                // Try to parse as identifier (could be a reference)
                let ident = self.parse_identifier()?;
                Ok(LuaValue::String(ident))
            }
        }
    }

    fn parse_string(&mut self) -> Result<LuaValue> {
        let quote = self
            .peek()
            .ok_or_else(|| WeakAuraError::LuaParseError("Expected string".to_string()))?;
        self.advance();

        let mut result = String::new();
        while let Some(c) = self.peek() {
            if c == quote {
                self.advance();
                break;
            }
            if c == '\\' {
                self.advance();
                match self.peek() {
                    Some('n') => {
                        result.push('\n');
                        self.advance();
                    }
                    Some('r') => {
                        result.push('\r');
                        self.advance();
                    }
                    Some('t') => {
                        result.push('\t');
                        self.advance();
                    }
                    Some('\\') => {
                        result.push('\\');
                        self.advance();
                    }
                    Some('"') => {
                        result.push('"');
                        self.advance();
                    }
                    Some('\'') => {
                        result.push('\'');
                        self.advance();
                    }
                    Some(c) => {
                        result.push(c);
                        self.advance();
                    }
                    None => break,
                }
            } else {
                result.push(c);
                self.advance();
            }
        }

        Ok(LuaValue::String(result))
    }

    fn parse_long_string(&mut self) -> Result<LuaValue> {
        // [[string]] or [=[string]=] etc.
        self.consume('[');
        let mut level = 0;
        while self.peek() == Some('=') {
            self.advance();
            level += 1;
        }
        self.consume('[');

        let mut result = String::new();
        loop {
            match self.peek() {
                Some(']') => {
                    let start_pos = self.pos;
                    self.advance();
                    let mut closing_level = 0;
                    while self.peek() == Some('=') {
                        self.advance();
                        closing_level += 1;
                    }
                    if self.peek() == Some(']') && closing_level == level {
                        self.advance();
                        break;
                    } else {
                        // Not the closing bracket, add to result
                        result.push(']');
                        for _ in 0..closing_level {
                            result.push('=');
                        }
                        self.pos = start_pos + 1 + closing_level;
                    }
                }
                Some(c) => {
                    result.push(c);
                    self.advance();
                }
                None => break,
            }
        }

        Ok(LuaValue::String(result))
    }

    fn parse_number(&mut self) -> Result<LuaValue> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.advance();
        }
        while let Some(c) = self.peek() {
            if c.is_ascii_digit()
                || c == '.'
                || c == 'e'
                || c == 'E'
                || c == '+'
                || c == '-'
                || c == 'x'
                || c == 'X'
                || c.is_ascii_hexdigit()
            {
                self.advance();
            } else {
                break;
            }
        }
        let num_str = &self.input[start..self.pos];
        let num: f64 = if num_str.contains('x') || num_str.contains('X') {
            // Hex number
            let hex_str = num_str.trim_start_matches("0x").trim_start_matches("0X");
            i64::from_str_radix(hex_str, 16).unwrap_or(0) as f64
        } else {
            num_str.parse().unwrap_or(0.0)
        };
        Ok(LuaValue::Number(num))
    }

    fn parse_identifier(&mut self) -> Result<String> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let ident = &self.input[start..self.pos];
        if ident.is_empty() {
            Err(WeakAuraError::LuaParseError(
                "Expected identifier".to_string(),
            ))
        } else {
            Ok(ident.to_string())
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comments(&mut self) {
        if self.peek() == Some('-') && self.peek_at(1) == Some('-') {
            self.advance();
            self.advance();
            // Check for long comment
            if self.peek() == Some('[')
                && (self.peek_at(1) == Some('[') || self.peek_at(1) == Some('='))
            {
                // Long comment [[...]] or [=[...]=]
                let mut level = 0;
                self.advance(); // [
                while self.peek() == Some('=') {
                    self.advance();
                    level += 1;
                }
                if self.peek() == Some('[') {
                    self.advance();
                    // Skip until closing
                    loop {
                        if self.peek() == Some(']') {
                            self.advance();
                            let mut closing_level = 0;
                            while self.peek() == Some('=') {
                                self.advance();
                                closing_level += 1;
                            }
                            if self.peek() == Some(']') && closing_level == level {
                                self.advance();
                                break;
                            }
                        } else if self.peek().is_some() {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
            } else {
                // Single line comment
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
            }
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.input[self.pos..].chars().nth(offset)
    }

    fn peek_identifier(&self) -> bool {
        matches!(self.peek(), Some(c) if c.is_alphabetic() || c == '_')
    }

    fn peek_word(&self, word: &str) -> bool {
        self.input[self.pos..].starts_with(word)
    }

    fn peek_long_string(&self) -> bool {
        if self.peek() != Some('[') {
            return false;
        }
        let rest = &self.input[self.pos + 1..];
        rest.starts_with('[') || rest.starts_with('=')
    }

    fn consume(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance(&mut self) {
        if let Some(c) = self.peek() {
            self.pos += c.len_utf8();
        }
    }

    fn advance_by(&mut self, n: usize) {
        for _ in 0..n {
            self.advance();
        }
    }
}
