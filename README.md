# WeakAura Mass Import

A Rust GUI application to mass import WeakAura strings directly into World of Warcraft's SavedVariables.

## Features

- **Parse** - Read multiple WeakAura import strings from text input, clipboard, or files
- **Decode** - View decoded WeakAura data as JSON
- **Validate** - Check if strings are valid WeakAura format (supports v0, v1, and v2 encoding)
- **Direct Import** - Write auras directly to WoW's `WeakAuras.lua` SavedVariables file

## Requirements

- Rust toolchain (1.70+)
- World of Warcraft installation (for importing to SavedVariables)

## Building

```bash
# Clone the repository
git clone <repo-url>
cd weakaura-mass-import

# Build release version
cargo build --release

# Run
cargo run --release
```

The built executable will be in `target/release/weakaura-mass-import.exe` (Windows) or `target/release/weakaura-mass-import` (Linux/macOS).

## Usage

1. **Set WoW Path**: Enter your World of Warcraft installation directory (auto-detected on common paths)
2. **Select SavedVariables**: Choose which account's WeakAuras.lua file to modify
3. **Input Auras**: 
   - Paste WeakAura strings directly (one per line)
   - Load from a text file
   - Paste from clipboard
4. **Parse**: Click "Parse" to decode and validate the strings
5. **Select**: Choose which auras to import
6. **Import**: Click "Import Selected" to write to SavedVariables

## Important Notes

- **Always backup your SavedVariables before importing!** The tool creates automatic backups (`.lua.backup`) but manual backups are recommended.
- **WoW must be closed** when importing to SavedVariables. Changes made while WoW is running will be overwritten.
- The tool supports all WeakAura encoding versions:
  - Version 0: Legacy format (LibCompress + AceSerializer)
  - Version 1: `!` prefix (LibDeflate + AceSerializer)
  - Version 2+: `!WA:2!` prefix (LibDeflate + LibSerialize)

## WeakAura String Format

WeakAura import strings are encoded as:
1. Version prefix (`!WA:2!`, `!`, or none)
2. Base64-like encoding (LibDeflate's custom alphabet)
3. DEFLATE compression
4. Binary serialization (LibSerialize for v2+, AceSerializer for older)

The tool uses the [weakauras-codec](https://crates.io/crates/weakauras-codec) crate for decoding.

## License

MIT

## Acknowledgments

- [WeakAuras2](https://github.com/WeakAuras/WeakAuras2) - The amazing addon this tool supports
- [weakauras-codec](https://github.com/Zireael-N/weakauras-codec-rs) - Rust implementation of WeakAura encoding/decoding
