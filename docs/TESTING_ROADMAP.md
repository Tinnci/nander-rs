# Testing Roadmap

## 🛡️ Current Testing Status

**Overall Status**: ✅ Strong Coverage for Logic; 🚧 Needs Real Hardware Automation.

### Existing Tests (Passing)

#### 1. Unit Tests (Domain & Application)
 Comprehensive unit testing covers the core business logic and use cases without requiring external dependencies.
- **Domain Layer**: 100% coverage of core types (`Address`, `Capacity`), logic (`BadBlock`, `ECC`), and traits.
- **Application Layer**: Mock-based testing for all primary Use Cases (`Read`, `Write`, `Erase`, `Verify`, `Detect`).

#### 2. Infrastructure Tests
- **NAND/NOR Protocols**: Validates packet formation, layout calculations, and spec access.
- **Simulated Programmer**: Verifies the in-memory SPI flash simulator itself.
- **Mock Programmer**: Validates the testing tooling.

#### 3. Integration Tests (Simulated)
- **E2E Lifecycle (`e2e_nand.rs`)**: A complete end-to-end integration test that simulates a physical SPI NAND chip.
    - **Scenario**: Connects `SpiNand` protocol to `SimulatedProgrammer`.
    - **Flow**: Erase Block -> Write Page -> Read Page -> Verify Content.
    - **Validation**: Ensures the entire software stack (Drivers -> Protocol -> App) functions correctly together.

---

## 🎯 Testing Priorities & Gaps

### High Priority: Automation & CI
The current tests run locally. We need to ensure they run on every commit.
- [ ] **GitHub Actions Workflow**: automated `cargo test` on Push/PR.
- [ ] **Clippy & FMT Check**: automated linting.

### Medium Priority: Real Hardware Verification
Simulation is perfect for logic, but cannot catch physical layer issues (timing, electrical noise, USB latency).
- [ ] **Hardware Test Script**: A script intended to be run by a developer with a specific reference chip (e.g., W25N01GV) connected to a CH341A.
- [ ] **Compatibility Matrix**: A manual test log verifying support for different manufacturers (Winbond, Kioxia, Macronix).

### Low Priority: Performance & Fuzzing
- [ ] **Benchmarks**: Measure throughput (MB/s) for different block sizes.
- [ ] **Fuzzing**: Feed garbage data to the packet parsers to ensure robustness.

---

## 🏗️ Testing Infrastructure

### The Simulator (`SimulatedProgrammer`)
We have built a custom SPI Flash Simulator in `src/infrastructure/programmer/simulator.rs`.
- **Capabilities**:
    - Simulates standard SPI NAND commands (0x02, 0x13, 0xD8, etc.).
    - Maintains internal state: Memory Array, Page Buffer, Status Register, Write Enable Latch.
    - Accurately models the page-load -> cache-read and program-load -> execute flows.
- **Usage**: Used in `tests/e2e_nand.rs` to validate the full stack.

### Mocking Framework
- Uses `mockall` for creating isolated unit tests for Use Cases.
- Allows testing error conditions (e.g., "Device Disconnected") that are hard to reproduce with real hardware.

---
*Last Updated: v0.4.1*