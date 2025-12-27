//! Tests for SPI NOR Flash Protocol
//!
//! These tests verify the NOR protocol implementation using mock programmers.

use crate::domain::bad_block::BadBlockStrategy;
use crate::domain::chip::{ChipCapabilities, ChipLayout, ChipSpec};
use crate::domain::types::{Capacity, FlashType, JedecId};
use crate::domain::{Address, FlashOperation, OobMode, Progress, ReadRequest};
use crate::infrastructure::flash_protocol::nor::SpiNor;
use crate::infrastructure::programmer::mock::MockProgrammer;
use crate::infrastructure::programmer::Programmer; // Import trait for method access
use std::cell::RefCell;

/// Create a test chip spec for a typical NOR flash (W25Q64)
fn create_test_nor_spec() -> ChipSpec {
    ChipSpec {
        name: "W25Q64".to_string(),
        manufacturer: "Winbond".to_string(),
        jedec_id: JedecId::new([0xEF, 0x40, 0x17]),
        flash_type: FlashType::Nor,
        capacity: Capacity::megabytes(8),
        layout: ChipLayout {
            page_size: 256,
            block_size: 64 * 1024, // 64KB
            oob_size: None,
        },
        capabilities: ChipCapabilities::default(),
    }
}

#[test]
fn test_nor_read_basic() {
    let mock = MockProgrammer::new();
    let spec = create_test_nor_spec();

    // Configure mock to return test data pattern
    // spi_transaction internally calls:
    // 1. spi_write (tx) -> calls spi_transfer, consumes one read response (discarded)
    // 2. spi_read (rx_len) -> calls spi_transfer, consumes one read response (returned)
    //
    // So we need to provide responses for BOTH transfers:
    mock.expect_read(vec![0xFF; 5]); // For spi_write (cmd+addr+dummy, 5 bytes), ignored
    let test_data = vec![0xDE, 0xAD, 0xBE, 0xEF];
    mock.expect_read(test_data.clone()); // For spi_read (actual data)

    let mut nor = SpiNor::new(mock, spec);

    // Create read request
    let request = ReadRequest {
        address: Address::new(0),
        length: 4,
        use_ecc: false,
        ignore_ecc_errors: false,
        oob_mode: OobMode::None,
        bad_block_strategy: BadBlockStrategy::Fail,
    };

    // Execute read
    let result = nor.read(request, &|_| {}).unwrap();

    // Verify data
    assert_eq!(result, vec![0xDE, 0xAD, 0xBE, 0xEF]);
}

#[test]
fn test_nor_read_progress_callback() {
    let mock = MockProgrammer::new();
    let spec = create_test_nor_spec();

    // Setup mock for chunked read - each chunk gets two spi_transfer calls
    // Chunk 1: spi_write (5 bytes) + spi_read_bulk (4095 bytes)
    mock.expect_read(vec![0xFF; 5]); // spi_write response (ignored)
    mock.expect_read(vec![0x00; 4095]); // spi_read_bulk response (first chunk data)
                                        // Chunk 2: spi_write (5 bytes) + spi_read_bulk (4095 bytes)
    mock.expect_read(vec![0xFF; 5]); // spi_write response (ignored)
    mock.expect_read(vec![0x00; 4095]); // spi_read_bulk response (second chunk data)

    let mut nor = SpiNor::new(mock, spec);

    let request = ReadRequest {
        address: Address::new(0),
        length: 8190, // Two chunks worth
        use_ecc: false,
        ignore_ecc_errors: false,
        oob_mode: OobMode::None,
        bad_block_strategy: BadBlockStrategy::Fail,
    };

    // Track progress calls using RefCell for interior mutability
    let progress_calls = RefCell::new(Vec::new());
    let result = nor.read(request, &|p: Progress| {
        progress_calls.borrow_mut().push((p.current, p.total));
    });

    assert!(result.is_ok());
    let calls = progress_calls.borrow();
    // Should have at least 2 progress updates
    assert!(
        calls.len() >= 2,
        "Expected at least 2 progress calls, got {}",
        calls.len()
    );
    // Last progress should show completion
    if let Some(last) = calls.last() {
        assert_eq!(last.0, last.1, "Final progress should show completion");
    }
}

#[test]
fn test_nor_spec_access() {
    let mock = MockProgrammer::new();
    let spec = create_test_nor_spec();
    let nor = SpiNor::new(mock, spec);

    // Verify spec access
    assert_eq!(nor.spec().name, "W25Q64");
    assert_eq!(nor.spec().flash_type, FlashType::Nor);
    assert_eq!(nor.spec().capacity.as_megabytes(), 8);
}

#[test]
fn test_mock_programmer_transactions() {
    let mut mock = MockProgrammer::new();

    // Verify CS control logging
    mock.set_cs(true).unwrap();
    assert!(mock.is_cs_active());
    mock.set_cs(false).unwrap();
    assert!(!mock.is_cs_active());

    // Verify transfers are logged
    mock.expect_read(vec![0xAB, 0xCD]);
    let mut rx = [0u8; 2];
    mock.spi_transfer(&[0x9F, 0x00], &mut rx).unwrap();
    assert_eq!(rx, [0xAB, 0xCD]);

    let writes = mock.get_writes();
    assert!(!writes.is_empty());
}
