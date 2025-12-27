# Testing Roadmap

## Current Testing Status

**Test Coverage: ~9.46% (537/5675 regions)**

### Existing Tests (33 unit tests passing)

#### Domain Layer Tests (12 tests)
- `types.rs`: 6 tests (Capacity, Address, JedecId, Progress, FlashOptions)
- `chip.rs`: 1 test (Layout calculations)
- `bad_block.rs`: 2 tests (Strategy logic, BadBlockTable operations)
- `ecc.rs`: 2 tests (Policy and Status helpers)
- `flash_operation.rs`: 1 test (Trait default implementations)

#### Application Layer Tests (8 tests)
- `read_flash.rs`: 1 test (Use case flow)
- `write_flash.rs`: 1 test (Use case flow)
- `erase_flash.rs`: 1 test (Use case flow)
- `verify_flash.rs`: 2 tests (Success and failure scenarios)
- `status_flash.rs`: 1 test (Get/Set status)
- `detect_chip.rs`: 2 tests (Known and unknown chip detection)

#### Infrastructure Layer Tests

**NAND Protocol Tests** (`src/infrastructure/flash_protocol/nand/tests.rs`): 4 tests
- `test_nand_spec_access` - Verifies chip specification access
- `test_nand_layout_calculations` - Tests page/block calculations
- `test_mock_programmer_spi_read_bulk` - Tests bulk read operations
- `test_mock_programmer_max_bulk_size` - Tests transfer size limits

**NOR Protocol Tests** (`src/infrastructure/flash_protocol/nor/tests.rs`): 4 tests
- `test_nor_read_basic` - Basic read operation test
- `test_nor_read_progress_callback` - Progress reporting test
- `test_nor_spec_access` - Specification access test
- `test_mock_programmer_transactions` - Mock programmer transaction test

**Mock Programmer Tests** (`src/infrastructure/programmer/mock.rs`): 3 tests
**EEPROM Tests** (`src/infrastructure/flash_protocol/eeprom/spi_25xxx.rs`): 2 tests

### Key Modules Needing Additional Tests

#### Application Layer
- `services/` - Pending implementation

#### Missing Integration Tests
- End-to-end flash operations
- Hardware integration tests
- Error handling scenarios
- Performance benchmarks

### Testing Infrastructure
- Uses `mockall` for mocking framework
- `MockProgrammer` for hardware simulation
- Test setup focuses on infrastructure layer only
- No domain or application layer test coverage

## Testing Priorities

### High Priority
1. **Domain Layer Tests** - Core business logic validation
2. **Error Handling Tests** - Comprehensive error scenario coverage
3. **Integration Tests** - End-to-end operation validation

### Medium Priority
1. **Application Layer Tests** - Use case validation
2. **Performance Tests** - Benchmark critical operations
3. **Hardware Compatibility Tests** - Real hardware validation

### Low Priority
1. **UI/CLI Tests** - Command-line interface testing
2. **Documentation Tests** - Example code validation

## Next Steps

1. Create test files for domain layer modules
2. Implement comprehensive error handling tests
3. Add integration test suite with mock hardware
4. Set up continuous integration testing
5. Add performance benchmarking framework