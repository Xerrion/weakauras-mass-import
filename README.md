# WeakAura Mass Import

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![Release](https://img.shields.io/github/v/release/Xerrion/weakauras-mass-import)](https://github.com/Xerrion/weakauras-mass-import/releases/latest)
[![CI](https://github.com/Xerrion/weakauras-mass-import/actions/workflows/ci.yml/badge.svg)](https://github.com/Xerrion/weakauras-mass-import/actions/workflows/ci.yml)

A fast Rust GUI for mass importing WeakAura strings directly into World of Warcraft's SavedVariables file — no addon manager needed.

## Features

- **Mass Import** — Parse and import multiple WeakAura strings at once from text input, files, or clipboard
- **Conflict Detection** — Detects existing auras and lets you selectively update only changed categories (Trigger, Load, Action, etc.)
- **Aura Removal** — Browse, select, and delete existing auras from SavedVariables via a tree view
- **Decode & Preview** — Inspect decoded aura data as JSON before importing
- **Auto-backup** — Always writes a `.lua.backup` before modifying your SavedVariables
- **All encoding versions** — Supports v0 (legacy), v1 (`!` prefix), and v2+ (`!WA:2!` prefix)
- **Toast notifications** — Non-blocking feedback for every operation

## Download

Grab the latest binary for your platform from the [Releases page](https://github.com/Xerrion/weakauras-mass-import/releases/latest):

| Platform | File |
|----------|------|
| Windows | `weakauras-mass-import-vX.Y.Z-x86_64-pc-windows-msvc.zip` |
| Linux | `weakauras-mass-import-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz` |
| macOS (Intel) | `weakauras-mass-import-vX.Y.Z-x86_64-apple-darwin.tar.gz` |
| macOS (Apple Silicon) | `weakauras-mass-import-vX.Y.Z-aarch64-apple-darwin.tar.gz` |

No installation required — extract and run.

## Usage

> **WoW must be closed** while importing. Any changes made while the game is running will be overwritten on exit.

1. **Select WoW install** — Point to your World of Warcraft directory; the app discovers available SavedVariables files automatically
2. **Select SavedVariables** — Choose which account's `WeakAuras.lua` to modify
3. **Add aura strings** — Paste strings directly, load a `.txt` file, or use the Paste button to pull from clipboard
4. **Parse** — Decodes and validates all strings, showing a preview of each aura
5. **Select** — Choose which auras to import (Select All / Deselect All available)
6. **Import** — Writes selected auras to SavedVariables; conflicts are surfaced for per-category resolution

### Managing Existing Auras

The sidebar shows all auras currently in your SavedVariables as a tree (groups with their children):

- **Expand/Collapse** groups to navigate
- **Select** individual auras or entire groups for removal
- **Remove** deletes them from SavedVariables (with automatic backup)

### Conflict Resolution

When importing an aura that already exists, the tool detects which categories changed and lets you choose per-aura:

- **Replace All** — overwrite the existing aura entirely
- **Update Selected** — merge only the categories you choose (e.g. Trigger only, leaving your custom Anchor untouched)
- **Skip** — leave the existing aura as-is

## Building from Source

Requires Rust 1.70+. Install via [rustup](https://rustup.rs).

```bash
git clone https://github.com/Xerrion/weakauras-mass-import.git
cd weakauras-mass-import

cargo build --release
# Binary: target/release/weakauras-mass-import(.exe)
```

### Development Commands

```bash
cargo build                         # Dev build
cargo run --release                 # Run with optimizations
cargo test                          # All tests
cargo test test_detect_version      # Single test by name
cargo test lua_parser_tests::       # All tests in a specific file
cargo fmt                           # Format code
cargo clippy --all-targets          # Lint
cargo run --bin decode_test -- <file_or_string>  # Inspect a decoded aura
```

## WeakAura String Encoding

WeakAura strings are structured as:

| Layer | v0 (legacy) | v1 (`!` prefix) | v2+ (`!WA:2!` prefix) |
|-------|-------------|-----------------|----------------------|
| Serialization | AceSerializer | AceSerializer | LibSerialize |
| Compression | LibCompress | LibDeflate | LibDeflate |
| Encoding | Custom base64 | Custom base64 | Custom base64 |

Decoding is handled by the [`weakauras-codec`](https://crates.io/crates/weakauras-codec) crate.

## Dependencies

| Crate | Purpose |
|-------|---------|
| [`iced`](https://github.com/iced-rs/iced) | Cross-platform GUI (0.14, tokio runtime) |
| [`weakauras-codec`](https://crates.io/crates/weakauras-codec) | WeakAura string decoding |
| [`full_moon`](https://crates.io/crates/full_moon) | Lua parsing for SavedVariables |
| [`rfd`](https://crates.io/crates/rfd) | Native file dialogs |
| [`arboard`](https://crates.io/crates/arboard) | Clipboard access |
| [`tokio`](https://crates.io/crates/tokio) | Async runtime |
| [`thiserror`](https://crates.io/crates/thiserror) | Error types |
| [`tracing`](https://crates.io/crates/tracing) | Logging |

## License

MIT — see [LICENSE](LICENSE).

## Acknowledgments

- [WeakAuras2](https://github.com/WeakAuras/WeakAuras2) — the addon this tool supports
- [weakauras-codec](https://crates.io/crates/weakauras-codec) — Rust implementation of WeakAura encoding/decoding
- [iced](https://github.com/iced-rs/iced) — cross-platform GUI library for Rust
