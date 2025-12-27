# nander-rs Project Roadmap

## 📊 Project Status (v0.5.0-rc)

`nander-rs` has completed all major features and is preparing for its first stable release on crates.io. The project has evolved from a direct SNANDer port to a production-ready, modern Rust flash programming suite.

| Module | Status | Completion | Notes |
|--------|--------|------------|-------|
| **Core Architecture** | ✅ Done | 100% | Layered Architecture (Domain/App/Infra/UI) |
| **Testing Suite** | ✅ Done | 95% | Unit Tests (Domain/App), E2E Simulation, Diagnostics |
| **CH341A Driver** | ✅ Done | 100% | Pure Rust `nusb`, Optimized Bulk Transfer, WinUSB Support |
| **SPI NAND** | ✅ Done | 100% | Read/Write/Erase, Bad Block Mgmt, OOB, ECC |
| **SPI NOR** | ✅ Done | 100% | Fast Read, 4-byte Address Mode |
| **EEPROM** | ✅ Done | 100% | I2C (24Cxx), SPI (25xxx), Microwire (93Cxx) |
| **Advanced Features** | ✅ Done | 100% | Auto-retry, Write Verification, BBT Scans, Speed Control |
| **GUI (egui)** | ✅ Done | 95% | Hex Preview, Drag-and-Drop, Read/Write/Erase, Real-time Logs |
| **Device Recognition** | ✅ Done | 100% | WCH Device Database, Intelligent Error Messages |
| **Documentation** | ✅ Done | 95% | User Guides, API Docs, Troubleshooting |

---

## 📅 Version Planning & Milestones

### v0.5.0: Ecosystem & Release (✅ READY FOR RELEASE)
**Goal**: Production-ready 1.0 candidate with complete documentation and crates.io publication.

- [x] **Release Engineering**
    - [x] Setup GitHub Actions (CI) for automated testing
    - [ ] Publish crate to crates.io (NEXT STEP)
    - [ ] Create binary releases for Windows/Linux/macOS
- [x] **Documentation**
    - [x] Comprehensive troubleshooting guides (WCH_DEVICES.md, WINDOWS_DRIVER_FIX.md)
    - [x] Testing guide without hardware (TESTING_WITHOUT_CHIP.md)
    - [x] User Guide with common usage examples
- [x] **GUI Implementation**
    - [x] Feature-complete UI with hex viewer
    - [x] Drag-and-drop support
    - [x] Real-time device diagnostics
- [x] **Developer Tools**
    - [x] Diagnostic CLI command for testing without chips
    - [x] Interactive SPI command tester

### v0.4.1: Testing & stability (✅ Completed)
**Focus**: Ensuring code correctness through comprehensive unit and integration testing.

- **Testing Infrastructure**:
    - [x] **Simulated Programmer**: In-memory SPI flash simulator for safe E2E testing without hardware.
    - [x] **Domain Layer Tests**: 100% coverage for core types (`Address`, `Capacity`, `JedecId`) and logic (`BadBlock`, `Ecc`).
    - [x] **Application Layer Tests**: Mock-based testing for all Use Cases (`Read`, `Write`, `Erase`, `Verify`, `Detect`).
    - [x] **E2E Integration**: Full lifecycle test (`Erase` -> `Write` -> `Read`) running against the simulator.

### v0.4.0: Advanced Features (✅ Released)
**Focus**: Reliability and Performance.

- **Performance**:
    - [x] CH341A Bulk SPI Read Optimization
    - [x] Single USB Transaction for CS+CMD+DATA
- **Reliability**:
    - [x] Automatic retry mechanism (`--retries`)
    - [x] Write verification (`--verify`)
- **New Commands**:
    - [x] `protect` / `status` management
    - [x] `bbt` (Bad Block Table) management
- **EEPROM Support**:
    - [x] I2C, Microwire, SPI EEPROM support

---

## 🔍 Feature Comparison (v0.4.x vs SNANDer)

| Feature | SNANDer (C) | nander-rs (Rust) | Advantage |
|---------|-------------|------------------|-----------|
| **Driver** | libusb (DLL) | **nusb (Native)** | Driverless, Plug-and-Play |
| **Performance** | Standard | **Extreme** | Bulk USB transfers, 90% fewer interactions |
| **Safety** | None | **High** | End-to-End Simulation, Heavy Unit Testing |
| **Architecture** | Monolithic | **DDD Layered** | Easy to extend and maintain |
| **UX** | Text | **Rich Interactive** | Progress bars, Colors, Time estimates |
| **Reliability** | Manual | **Auto-Retry** | Critical for aging chips |

---

## 🛠 TODO List

### Long Term
1. **GUI Client Enhancement**
   - Hex editor integration.
   - Batch programming support.

2. **Broad Hardware Support**
   - Native linux `spidev` support.
   - FTDI (FT232H/2232H) support.

3. **Scripting**
   - Lua/Python scripting for custom chip protocols.

---

*Last Updated: 2025-12-27 (v0.5.0-dev)*
