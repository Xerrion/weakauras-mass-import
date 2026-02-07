# AGENTS.md - WeakAura Mass Import

Rust GUI application for mass importing WeakAura strings into WoW SavedVariables files.

**Stack**: Rust 1.70+ (MSRV), iced 0.14 (tokio), weakauras-codec

## Build & Test Commands

```bash
cargo build              # Dev build
cargo build --release    # Optimized (LTO enabled)
cargo run --release      # Run application

# Tests
cargo test                              # All tests
cargo test test_detect_version          # Single test by name
cargo test lua_parser_tests::           # Tests in specific file (tests/lua_parser_tests.rs)
cargo test -- --nocapture               # Show println output

# Quality
cargo fmt                               # Format (CI enforces)
cargo clippy --all-targets --all-features  # Lint (CI treats warnings as errors)
cargo check --release                   # Fast release check

# Debug utility
cargo run --bin decode_test -- <file_or_string>
```

## Project Structure

```
src/
├── main.rs              # Entry point
├── lib.rs               # Public API exports
├── app/                 # GUI (iced)
│   ├── mod.rs           # App state, Message enum, update()
│   ├── state.rs         # Shared state types (ParsedAuraEntry, etc.)
│   ├── actions/         # Business logic
│   │   ├── handlers.rs  # Message handlers
│   │   ├── import.rs    # Import flow
│   │   ├── loading.rs   # Aura parsing
│   │   └── removal.rs   # Aura removal
│   └── ui/              # Rendering
│       ├── main_panel.rs
│       ├── sidebar.rs
│       └── dialogs.rs
├── decoder.rs           # WeakAura string decoding, LuaValue types
├── lua_parser.rs        # SavedVariables parsing/serialization
├── saved_variables.rs   # SavedVariables management, conflict detection
├── categories.rs        # Update category mapping
├── theme.rs             # Cyber Dark 2026 theme (colors, typography, spacing)
├── util.rs              # Utility functions
└── error.rs             # Error types
tests/
├── decoder_tests.rs
├── lua_parser_tests.rs
├── saved_variables_tests.rs
├── categories_tests.rs
└── integration_test.rs
```

## Code Style

### Imports (3 groups, alphabetical within)
```rust
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use iced::widget::{button, column, container};
use serde::{Deserialize, Serialize};

use crate::decoder::{LuaValue, WeakAura};
use crate::error::{Result, WeakAuraError};
```

### Naming
- Types: `PascalCase` (`WeakAura`, `LuaValue`, `ConflictAction`)
- Functions: `snake_case` (`decode_multiple`, `parse_input`)
- Constants: `SCREAMING_SNAKE_CASE` (`TRIGGER_FIELDS`)

### Error Handling
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeakAuraError {
    #[error("Lua parse error: {0}")]
    LuaParseError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),  // #[from] for automatic conversion
}

pub type Result<T> = std::result::Result<T, WeakAuraError>;
```
- Prefer `?` operator
- Use `.map_err()` for conversions
- Add `#[from]` when wrapping external errors

### Structs & Enums
- Derive `Debug, Clone` for data types
- Add `Serialize, Deserialize` for persistence/JSON
- Use `#[derive(Default)]` where sensible
- `#[serde(untagged)]` for variant types like `LuaValue`
- `#[derive(Copy)]` for small enums without heap data

### Documentation
- `//!` module-level docs at file top
- `///` for public items

### GUI (iced 0.14)
- Render methods in `app/ui/`: `render_sidebar()`, `render_main_content()`
- Theme in `theme.rs`: `colors::GOLD`, `typography::HEADING`, `spacing::MD`
- Containers: `container(...).style(theme::container_panel)`
- Buttons: `button(...).style(theme::button_primary)`

### Tests
- External test files in `tests/` directory
- Name: `test_<function>_<scenario>`
- Test edge cases: empty input, invalid data, boundary conditions

## Key Types

### LuaValue
Lua data representation:
- `Nil`, `Bool(bool)`, `Number(f64)`, `String(String)`
- `Array(Vec<LuaValue>)` - numeric indices
- `Table(HashMap<String, LuaValue>)` - string keys
- `MixedTable { array, hash }` - both (common in WeakAuras)

### WeakAura
Decoded aura: `id`, `uid`, `region_type`, `is_group`, `children`, `child_data`, `data: LuaValue`, `encoding_version`

### UpdateCategory
Selective update categories: `Trigger`, `Load`, `Action`, `Animation`, `Conditions`, `Anchor`, etc.

## Critical Behaviors

1. **MixedTable serialization**: Array elements use implicit indices (`{ }, -- [1]`), NOT string keys (`["1"]`)
2. **Conflict detection**: On existing aura import, detect changed categories for selective updates
3. **SavedVariables format**: `WeakAurasSaved.displays` table contains all auras
4. **Backups**: Always create `.lua.backup` before modifying

## Common Tasks

### Adding UpdateCategory
1. Add variant in `categories.rs` `UpdateCategory` enum
2. Add to `display_name()`, `default_enabled()`, `all()`
3. Add fields to `*_FIELDS` constant
4. Update `get_category()` match

### Adding GUI component
1. Create render method in appropriate `app/ui/*.rs`
2. Use theme: `colors::*`, `typography::*`, `spacing::*`
3. Add `Message` variant in `app/mod.rs` if needed
4. Handle in `update()` match

### Adding error variant
1. Add to `WeakAuraError` in `error.rs`
2. Use `#[error("...")]` for message
3. Use `#[from]` if wrapping another error type

## CI Requirements
- `cargo fmt --check` must pass
- `cargo clippy` with `-Dwarnings` must pass
- Tests must pass on: ubuntu, windows, macos × stable, 1.70 (MSRV)
