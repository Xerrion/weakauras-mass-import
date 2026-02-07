# AGENTS.md - WeakAura Mass Import

## Project Overview

A Rust GUI application for mass importing WeakAura strings into World of Warcraft's SavedVariables files. Built with **iced 0.14** for the GUI (Cyber Dark 2026 theme), using the `weakauras-codec` crate for decoding WeakAura strings.

**Tech Stack**: Rust 1.70+, iced 0.14 (tokio), weakauras-codec

## Build & Run Commands

```bash
# Development build
cargo build

# Release build (optimized, LTO enabled)
cargo build --release

# Run application
cargo run --release

# Run all tests
cargo test

# Run a single test by name
cargo test test_detect_version
cargo test test_parse_mixed_table

# Run tests in a specific module
cargo test decoder::tests
cargo test lua_parser::tests
cargo test categories::tests

# Run tests with output shown
cargo test -- --nocapture

# Check code without building
cargo check

# Format code
cargo fmt

# Lint with clippy
cargo clippy

# Run the decode_test binary (debugging helper)
cargo run --bin decode_test -- <file_or_string>
```

## Project Structure

```
src/
  main.rs           # Entry point, egui initialization
  lib.rs            # Library exports for public API
  app.rs            # Main GUI application state and rendering (~1400 lines)
  decoder.rs        # WeakAura string decoding, LuaValue types
  lua_parser.rs     # SavedVariables file parsing and serialization
  saved_variables.rs # SavedVariables management, conflict detection
  categories.rs     # Update category mapping (triggers, load, etc.)
  theme.rs          # Dark theme with WoW gold accents
  error.rs          # Custom error types
  bin/
    decode_test.rs  # Debug utility for inspecting decoded auras
```

## Code Style Guidelines

### Imports
- Group imports: std first, then external crates, then local modules
- Use `use crate::` for internal imports
- Sort alphabetically within groups

```rust
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::decoder::{LuaValue, WeakAura};
use crate::error::{Result, WeakAuraError};
```

### Naming Conventions
- Types: `PascalCase` (e.g., `WeakAura`, `LuaValue`, `ImportConflict`)
- Functions/methods: `snake_case` (e.g., `decode_multiple`, `parse_input`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `TRIGGER_FIELDS`)
- Module-level const arrays: `&'static [&'static str]` pattern

### Error Handling
- Use `thiserror` for error definitions
- Custom `Result<T>` type alias: `pub type Result<T> = std::result::Result<T, WeakAuraError>;`
- Prefer `?` operator for propagation
- Use `.map_err()` for error conversion
- Match on error variants when needed for different handling

```rust
// Error definition pattern
#[derive(Error, Debug)]
pub enum WeakAuraError {
    #[error("Invalid WeakAura string: {0}")]
    InvalidString(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

### Documentation
- Use `//!` for module-level docs at top of file
- Use `///` for public items
- Include examples in doc comments where helpful

### Struct Patterns
- Derive `Debug, Clone` for most data types
- Add `Serialize, Deserialize` for types that need JSON/persistence
- Use `#[derive(Default)]` where sensible
- Mark unused code with `#[allow(dead_code)]` if intentional

### Enums
- Use `#[serde(untagged)]` for JSON variants like `LuaValue`
- Implement `Display` for user-facing enums
- Use `#[derive(Copy)]` for small enums without heap data

### GUI Patterns (iced)
- Separate render methods: `render_menu_bar()`, `render_sidebar()`, etc.
- Use `container()` for visual sections with `theme::container_*` styles
- Apply theme colors from `theme::colors::*`
- Use `text().size(typography::*)` for styled text

### Testing
- Test module at end of file: `#[cfg(test)] mod tests { ... }`
- Use `use super::*;` in test modules
- Name tests descriptively: `test_<function>_<scenario>`
- Test edge cases: empty input, invalid data, boundary conditions

## Key Types

### LuaValue
Represents Lua data (since WeakAura data is Lua tables):
- `Nil`, `Bool`, `Number`, `String`
- `Array` - contiguous numeric indices
- `Table` - string-keyed hash map
- `MixedTable { array, hash }` - both (common in WeakAuras triggers)

### WeakAura
Decoded aura with:
- `id`, `uid`, `region_type`
- `is_group`, `children`, `child_data`
- `data: LuaValue`, `encoding_version`

### UpdateCategory
Categories for selective updates: `Trigger`, `Load`, `Action`, `Animation`, `Conditions`, `Anchor`, etc.

## Critical Behaviors

1. **Mixed Tables**: WeakAuras triggers use Lua tables with both numeric and string keys. The `MixedTable` variant preserves this - array elements use implicit indices (no `["1"]`), hash keys use explicit brackets.

2. **Serialization**: When serializing back to Lua, array elements must NOT have string numeric keys like `["1"]`. Use implicit indices with comments: `{ }, -- [1]`

3. **Conflict Detection**: When importing existing auras, detect which categories changed and allow selective updates.

4. **SavedVariables Format**: WoW's `WeakAuras.lua` file assigns to `WeakAurasSaved` global with `displays` table containing all auras.

## Dependencies

Key crates:
- `iced` - GUI framework (0.14)
- `weakauras-codec` - Decoding WeakAura strings
- `serde`/`serde_json` - Serialization
- `full_moon` - Lua parsing (in Cargo.toml but lua_parser is custom)
- `rfd` - Native file dialogs
- `arboard` - Clipboard access
- `anyhow`/`thiserror` - Error handling
- `tracing` - Logging

## Common Tasks

### Adding a new UpdateCategory
1. Add variant to `UpdateCategory` enum in `categories.rs`
2. Add display name in `display_name()`
3. Add fields to appropriate `*_FIELDS` constant
4. Update `get_category()` match arm

### Modifying GUI
1. Find relevant `render_*` method in `app.rs`
2. Use theme colors from `theme::colors`
3. Follow existing patterns for frames, buttons, labels

### Adding error variants
1. Add variant to `WeakAuraError` in `error.rs`
2. Use `#[error("...")]` attribute for message
3. Add `#[from]` for automatic conversion if wrapping another error

## Notes

- WoW must be closed when importing (game overwrites SavedVariables)
- Tool creates `.lua.backup` before modifying
- Supports encoding versions 0, 1, and 2+ (different compression/serialization)
- The `Addons/` folder contains reference WeakAuras addon Lua files (not part of this tool's code)
