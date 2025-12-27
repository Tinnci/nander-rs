# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.3] - 2025-12-27

### Fixed
- **Release Automation**: Fixed automated release builds on Windows/Linux/macOS.
- **CH347 Protocol**: Corrected critical protocol errors in SPI configuration and CS control (v0.5.1).

### Added
- **Multi-Platform Releases**: CI now automatically builds and attaches binaries for Windows, Linux, and macOS to GitHub Releases.
- **Trusted Publishing**: Switched release workflow to use OIDC authentication for improved security.

## [0.5.0] - 2025-12-27

### Added
- **CH347 High-Speed Programmer Support**
  - Full support for CH347 (Mode 1) with SPI speeds up to 60MHz
  - Automatic identification of CH347 series
  - Optimized USB communication packet builders for CH347
- **Automated Batch Programming Mode**
  - New `batch` command for sequential flash operations
  - Support for JSON and TOML batch scripts
  - Built-in templates for common workflows (`flash-update`, `production`)
  - Integration with all supported flash types (NAND, NOR, EEPROM)
  - Automatic delay and retry logic in automated flows
- **Cross-platform GUI** using `egui` framework
  - Real-time hex viewer with virtual scrolling for large files
  - Drag-and-drop file support for easy firmware loading
  - Live device detection and status monitoring
  - Progress bars for read/write/erase operations
  - Expandable log viewer with timestamps
- **WCH Device Recognition System**
  - Comprehensive device database for all WCH/QinHeng USB chips
  - Intelligent mode detection (SPI vs UART for CH341)
  - Automatic troubleshooting guidance for wrong-mode devices
  - Support for CH341, CH347, CH348, CH340, CH9102 series recognition
- **Windows Driver Support**
  - Automatic detection of CH341A driver issues
  - Step-by-step WinUSB installation guide with Zadig
  - Clear error messages with actionable solutions
- **Diagnostic Tools**
  - `nander diagnostic` command for testing without flash chip
  - USB communication verification
  - SPI bus status testing
  - GPIO control test (LED blink)
  - JEDEC ID reading capability check
  - Interactive SPI command tester (`--interactive` flag)
- **Comprehensive Documentation**
  - WCH_DEVICES.md - Complete device compatibility guide
  - WINDOWS_DRIVER_FIX.md - Quick Windows driver fix guide
  - TESTING_WITHOUT_CHIP.md - Testing guide for users without hardware
  - TESTING_ROADMAP.md - Testing strategy and infrastructure
  - PERFORMANCE_OPTIMIZATION.md - Optimization details
  - BBT_DESIGN.md - Bad block table implementation
  - EEPROM_USAGE.md - EEPROM programming guide

### Improved
- **Enhanced Error Messages**
  - Context-aware error reporting
  - Automatic detection of common issues (driver, mode, connection)
  - Suggested solutions for each error type
- **CH341A Driver Optimization**
  - Bulk transfer optimization (90% reduction in USB overhead)
  - Larger packet sizes for flash operations (up to 4KB)
  - Smart switching between bulk and standard transfer modes
- **Architecture Refinement**
  - Domain-Driven Design with clear layer separation
  - Improved trait abstractions for `Programmer` interface
  - Better separation of concerns between CLI and GUI
- **Progress Reporting**
  - More granular progress updates
  - Percentage-based feedback in both CLI and GUI
  - Estimated time remaining (CLI only)

### Fixed
- Windows WinUSB compatibility issues with clear resolution steps
- CH341 device mode confusion (UART mode auto-detected and explained)
- Memory efficiency for large flash read operations
- Clippy warnings and code quality issues
- Input validation for hex/decimal address parsing in GUI

### Changed
- Version bumped from 0.4.0 to 0.5.0
- Minimum supported Rust version: 1.87
- Default SPI speed increased to 3MHz (was variable)
- GUI read operations now also populate hex preview

## [0.4.0] - 2025-12-26

### Added
- Full SPI NAND support with ECC and bad block management
- SPI NOR flash support
- I2C EEPROM (24Cxx series) support
- SPI EEPROM (25xxx series) support
- Microwire EEPROM (93Cxx series) support
- Comprehensive test suite with E2E simulation
- CH341A programmer support via pure Rust `nusb`

### Improved
- Migrated from SNANDer C codebase to modern Rust architecture
- Implemented layered architecture (Domain/Application/Infrastructure/Presentation)
- Added extensive unit and integration tests

## [0.3.0] - 2025-12-26

### Added
- Initial Rust implementation
- Basic flash operations (read/write/erase)
- CLI interface with `clap`

---

[Unreleased]: https://github.com/tinnci/nander-rs/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/tinnci/nander-rs/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/tinnci/nander-rs/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/tinnci/nander-rs/releases/tag/v0.3.0
