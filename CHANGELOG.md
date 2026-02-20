# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [3.1.1](https://github.com/Xerrion/weakauras-mass-import/compare/weakauras-mass-import-v3.1.0...weakauras-mass-import-v3.1.1) (2026-02-20)


### Documentation

* expand AGENTS.md with arch, git workflow, and app/ subguide ([#8](https://github.com/Xerrion/weakauras-mass-import/issues/8)) ([15fc80f](https://github.com/Xerrion/weakauras-mass-import/commit/15fc80f07e6592d33b6cb5dc1e8c5a59bd33eda2))

## [3.1.0](https://github.com/Xerrion/weakauras-mass-import/compare/weakauras-mass-import-v3.0.0...weakauras-mass-import-v3.1.0) (2026-02-19)


### Features

* implement WeakAuras-like aura deletion and UI improvements ([#7](https://github.com/Xerrion/weakauras-mass-import/issues/7)) ([e796388](https://github.com/Xerrion/weakauras-mass-import/commit/e7963888b73f7e6c8b41eeb7e127b80ec20564e5))


### Code Refactoring

* reorganize WeakAuraImporter with nested state structs ([51d5e63](https://github.com/Xerrion/weakauras-mass-import/commit/51d5e636fec1f7ba4d079af10546a392120b8914))
* reorganize WeakAuraImporter with nested state structs ([7c7838f](https://github.com/Xerrion/weakauras-mass-import/commit/7c7838feb71c90e293980f5ee806f18eb3dac38b))


### Documentation

* update README and AGENTS.md to reflect current architecture ([fd1ad16](https://github.com/Xerrion/weakauras-mass-import/commit/fd1ad167c94d3b1f3589c79c4f11b6af5ba71210))
* update README and AGENTS.md to reflect current architecture ([633610c](https://github.com/Xerrion/weakauras-mass-import/commit/633610c90709a4e0159cfb98bd8e9c5190798b5f))

## [3.0.0](https://github.com/Xerrion/weakauras-mass-import/compare/weakauras-mass-import-v2.0.0...weakauras-mass-import-v3.0.0) (2026-02-07)


### ⚠ BREAKING CHANGES

* Complete rewrite of GUI layer from egui/eframe to iced-rs

### Features

* add toast notifications and setup wizard ([1ad3c64](https://github.com/Xerrion/weakauras-mass-import/commit/1ad3c64f3f225bb4a127b93023cc51c49effb84b))
* migrate GUI framework from egui to iced 0.14 ([1828f48](https://github.com/Xerrion/weakauras-mass-import/commit/1828f48689dd44905b373441a1aae8a36edf4d24))
* modernize UI with comprehensive design system ([dd3deed](https://github.com/Xerrion/weakauras-mass-import/commit/dd3deeddfc5164d62bdaf249c586296c29b10d8c))


### Bug Fixes

* status bar shows only import progress and updates dynamically ([d688b4c](https://github.com/Xerrion/weakauras-mass-import/commit/d688b4c9e4c727efa09a2835b8e4272d6db1592e))


### Code Refactoring

* split actions.rs into focused submodules ([6b7252f](https://github.com/Xerrion/weakauras-mass-import/commit/6b7252ff90967c38e5a16367e239c3d3ab091bbb))

## [2.0.0](https://github.com/Xerrion/weakauras-mass-import/compare/weakauras-mass-import-v1.0.0...weakauras-mass-import-v2.0.0) (2026-02-07)


### ⚠ BREAKING CHANGES

* Package renamed from weakaura-mass-import to weakauras-mass-import

### Features

* add aura removal feature with confirmation dialog ([ecb4981](https://github.com/Xerrion/weakauras-mass-import/commit/ecb498176211b8e59b01e10a164efb5546fce81b))
* add bulk Remove Selected button for loaded auras ([f39e54a](https://github.com/Xerrion/weakauras-mass-import/commit/f39e54a8ab962d91bcd69664b59de2d65602d21e))
* add decode_auras_filtered helper for duplicate detection ([ec0a214](https://github.com/Xerrion/weakauras-mass-import/commit/ec0a2142dab1fa9d574ede2d2b113e4e0de92edb))
* add individual X remove button per loaded aura row ([f510702](https://github.com/Xerrion/weakauras-mass-import/commit/f5107027863e6d49e256f5eb050fb21e7a3233c1))
* add poll_loading to process async results ([12c5910](https://github.com/Xerrion/weakauras-mass-import/commit/12c591073ac91f3e350372d0fd7fa88a2cf61dce))
* add release system, GitHub Actions CI/CD, and optimize Cargo.toml ([218ad9d](https://github.com/Xerrion/weakauras-mass-import/commit/218ad9d4621625b0b525ffefc1183afcee0c563f))
* implement async file loading with progress ([6cfe8db](https://github.com/Xerrion/weakauras-mass-import/commit/6cfe8dbbd1983df6b533066f7ec415fe00730819))
* implement async folder loading with progress ([cc46434](https://github.com/Xerrion/weakauras-mass-import/commit/cc46434235959b1839427f91a004f11e0b7b8103))
* initial WeakAura mass import tool ([1d4d896](https://github.com/Xerrion/weakauras-mass-import/commit/1d4d8962ff1bceb5820a3e0c400f0b2c34a74a91))
* make aura removal async with progress indicator ([40b1d82](https://github.com/Xerrion/weakauras-mass-import/commit/40b1d82e4c18d93661bbceebb1fb376140069dfe))
* make import and SavedVariables loading async with progress indicators ([02bb4ef](https://github.com/Xerrion/weakauras-mass-import/commit/02bb4ef02d220508f0c262dfd7246ed7f88734f4))
* render loading progress bar and disable buttons during load ([88fd2fb](https://github.com/Xerrion/weakauras-mass-import/commit/88fd2fba06d18ce43442b406b4f61b48cb3ae5bc))


### Bug Fixes

* handle boolean/nil array elements and special number values in Lua parser ([678f771](https://github.com/Xerrion/weakauras-mass-import/commit/678f771528bb0a3362cb2a4a49e1553bc88c9e87))
* make sidebar checkboxes always visible with custom painting ([02dcefd](https://github.com/Xerrion/weakauras-mass-import/commit/02dcefdd89a30d32765e40cdcfc869fbac370439))
* skip integration tests requiring local fixtures in CI ([9d43925](https://github.com/Xerrion/weakauras-mass-import/commit/9d43925d48c86fa69da25097a7f77c244b3ec131))
* tree display to recursively nest subgroups ([e68081e](https://github.com/Xerrion/weakauras-mass-import/commit/e68081e3253c4d94cfee0f792a55b5dd1e36e8be))


### Code Refactoring

* add Clone to ParsedAuraEntry, add LoadingUpdate enum ([56aa94e](https://github.com/Xerrion/weakauras-mass-import/commit/56aa94eb920e2cb8d044b2a3ab64fa6a83f2984d))
* add loading state fields and tokio runtime ([c0da631](https://github.com/Xerrion/weakauras-mass-import/commit/c0da631b73442aaf4f0472f4dd24c44b36030a7d))
* extract shared helpers into util.rs ([f22593b](https://github.com/Xerrion/weakauras-mass-import/commit/f22593b98e70713b18cd4d0cff4453d50ba33014))
* move inline unit tests to tests directory ([fbb26dc](https://github.com/Xerrion/weakauras-mass-import/commit/fbb26dcb650bb6b99c20205106f273117a343bfe))
* restructure app.rs into module tree ([6586149](https://github.com/Xerrion/weakauras-mass-import/commit/658614934c9419c56e14379edc7e28ff7efd0513))


### Miscellaneous

* add tokio dependency ([107d063](https://github.com/Xerrion/weakauras-mass-import/commit/107d0638d16a7c25a036ca3b8b45cfbc63514a9f))
* fix clippy warnings and apply cargo fmt ([9db3236](https://github.com/Xerrion/weakauras-mass-import/commit/9db323688c5d17187357bac6286f4533766c3afc))

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
