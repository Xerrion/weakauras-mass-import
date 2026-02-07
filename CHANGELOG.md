# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0](https://github.com/Xerrion/weakauras-mass-import/releases/tag/v1.0.0) (2026-02-07)

### Features

* Initial release of WeakAuras Mass Import tool
* **GUI Application**: Modern egui-based interface with WoW-themed dark mode
* **Mass Import**: Parse and import multiple WeakAura strings at once
* **Direct SavedVariables Editing**: Write directly to WoW's WeakAuras.lua file
* **Encoding Support**: Full support for WeakAura encoding versions 0, 1, and 2+
* **Hierarchy Preservation**: Correctly handles nested groups and subgroups
* **Conflict Detection**: Detect existing auras and choose update categories
* **Async Operations**: Non-blocking file loading with progress indicators
* **Aura Management**: Add, remove, and manage auras with bulk operations
* **Automatic Backups**: Creates .lua.backup before modifying SavedVariables

### Technical Highlights

* Built with Rust for performance and reliability
* Uses `weakauras-codec` crate for decoding
* Custom Lua parser for SavedVariables with full_moon AST
* Cross-platform support (Windows, Linux, macOS)
