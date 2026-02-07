# WeakAura Mass Import

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)](https://www.rust-lang.org/)

Mass import WeakAura strings into World of Warcraft SavedVariables with a fast Rust GUI. Built with **iced 0.14** and the `weakauras-codec` crate for decoding.

## Features

- **Mass Import** - Parse multiple WeakAura strings from text input, clipboard, or files
- **Decode & Preview** - View decoded WeakAura data as JSON
- **Validate** - Check if strings are valid WeakAura format (supports v0, v1, and v2+ encoding)
- **Direct Import** - Write auras directly to WoW's `WeakAuras.lua` SavedVariables file
- **Conflict Detection** - Detect existing auras and selectively update specific categories
- **Aura Removal** - Browse and remove existing auras from SavedVariables
- **Tree View** - Hierarchical view of existing auras (groups and children)
- **Auto-backup** - Creates `.lua.backup` before any modifications
- **Toast Notifications** - Visual feedback for all operations

## Requirements

- Rust toolchain (1.70+)
- World of Warcraft installation (for importing to SavedVariables)

## Installation

```bash
# Clone the repository
git clone https://github.com/Xerrion/weakauras-mass-import.git
cd weakauras-mass-import

# Release build (optimized)
cargo build --release

# Run application
cargo run --release
```

The built executable will be in `target/release/weakauras-mass-import.exe` (Windows) or `target/release/weakauras-mass-import` (Linux/macOS).

## Usage

1. **Select Install** - Choose your World of Warcraft installation directory
2. **Select SavedVariables** - Pick which account's WeakAuras.lua file to modify
3. **Input Auras**:
   - Paste WeakAura strings directly (one per line)
   - Load from a text file via **File** button
   - Paste from clipboard via **Paste** button
4. **Parse** - Click "Parse" to decode and validate the strings
5. **Select** - Choose which auras to import (use Select All/Deselect All)
6. **Import** - Click "Import Selected" to write to SavedVariables

### Managing Existing Auras

The sidebar displays all existing auras in a tree structure:
- **Expand/Collapse** - Toggle group visibility
- **Select/Deselect** - Mark auras for removal
- **Remove** - Delete selected auras from SavedVariables

## Development

```bash
# Development build
cargo build

# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Check code without building
cargo check

# Format code
cargo fmt

# Lint with clippy
cargo clippy

# Run the decode_test utility
cargo run --bin decode_test -- <file_or_string>
```

## Project Structure

```
src/
├── main.rs              # Entry point
├── lib.rs               # Library exports
├── app/                 # GUI application (iced)
│   ├── mod.rs           # Main app state and message handling
│   ├── state.rs         # Shared state types
│   ├── actions/         # Business logic handlers
│   │   ├── handlers.rs  # Message handlers
│   │   ├── import.rs    # Import logic
│   │   ├── loading.rs   # Aura parsing/loading
│   │   └── removal.rs   # Aura removal logic
│   └── ui/              # UI rendering components
│       ├── main_panel.rs  # Main content area
│       ├── sidebar.rs     # Existing auras tree
│       └── dialogs.rs     # Confirmation dialogs
├── decoder.rs           # WeakAura string decoding
├── lua_parser.rs        # SavedVariables parsing/serialization
├── saved_variables.rs   # SavedVariables management, conflict detection
├── categories.rs        # Update category mapping
├── theme.rs             # Cyber Dark 2026 theme with WoW gold accents
├── util.rs              # Utility functions
├── error.rs             # Custom error types
└── bin/
    └── decode_test.rs   # Debug utility for inspecting decoded auras
tests/
├── integration_test.rs
├── decoder_tests.rs
├── lua_parser_tests.rs
├── saved_variables_tests.rs
└── categories_tests.rs
```

## Important Notes

- **Always backup your SavedVariables!** The tool creates automatic backups (`.lua.backup`) but manual backups are recommended.
- **WoW must be closed** when importing. Changes made while WoW is running will be overwritten when you exit the game.
- Supports all WeakAura encoding versions:
  - **Version 0**: Legacy format (LibCompress + AceSerializer)
  - **Version 1**: `!` prefix (LibDeflate + AceSerializer)
  - **Version 2+**: `!WA:2!` prefix (LibDeflate + LibSerialize)

## WeakAura String Format

WeakAura import strings are encoded as:

1. Version prefix (`!WA:2!`, `!`, or none)
2. Base64-like encoding (LibDeflate's custom alphabet)
3. DEFLATE compression
4. Binary serialization (LibSerialize for v2+, AceSerializer for older)

The tool uses the [weakauras-codec](https://crates.io/crates/weakauras-codec) crate for decoding.

## Dependencies

| Crate | Purpose |
|-------|---------|
| `iced` | GUI framework (0.14 with tokio) |
| `iced_toasts` | Toast notifications |
| `weakauras-codec` | WeakAura string decoding |
| `full_moon` | Lua parsing for SavedVariables |
| `rfd` | Native file dialogs |
| `arboard` | Clipboard access |
| `serde` / `serde_json` | Serialization |
| `tokio` | Async runtime |
| `tracing` | Logging |

## License

MIT

## Acknowledgments

- [WeakAuras2](https://github.com/WeakAuras/WeakAuras2) - The amazing addon this tool supports
- [weakauras-codec](https://crates.io/crates/weakauras-codec) - Rust implementation of WeakAura encoding/decoding
- [iced](https://github.com/iced-rs/iced) - Cross-platform GUI library for Rust
