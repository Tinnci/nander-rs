//! Tests for SPI NAND Flash Protocol
//!
//! These tests verify the NAND protocol implementation using mock programmers.

use crate::domain::chip::{ChipCapabilities, ChipLayout, ChipSpec};
use crate::domain::types::{Capacity, FlashType, JedecId};
use crate::infrastructure::flash_protocol::nand::SpiNand;
use crate::infrastructure::programmer::mock::MockProgrammer;
use crate::infrastructure::programmer::Programmer; // Import trait for method access

/// Create a test chip spec for a typical NAND flash (W25N01GV)
fn create_test_nand_spec() -> ChipSpec {
    ChipSpec {
        name: "W25N01GV".to_string(),
        manufacturer: "Winbond".to_string(),
        jedec_id: JedecId::new([0xEF, 0xAA, 0x21]),
        flash_type: FlashType::Nand,
        capacity: Capacity::megabytes(128),
        layout: ChipLayout {
            page_size: 2048,
            block_size: 128 * 1024, // 128KB (64 pages per block)
            oob_size: Some(64),
            is_dataflash: false,
        },
        capabilities: ChipCapabilities {
            supports_ecc_control: true,
            ..Default::default()
        },
        otp: None,
    }
}

#[test]
fn test_nand_spec_access() {
    let mock = MockProgrammer::new();
    let spec = create_test_nand_spec();
    let nand = SpiNand::new(mock, spec);

    // Verify spec access
    assert_eq!(nand.spec().name, "W25N01GV");
    assert_eq!(nand.spec().flash_type, FlashType::Nand);
    assert_eq!(nand.spec().capacity.as_megabytes(), 128);
    assert_eq!(nand.spec().layout.page_size, 2048);
    assert_eq!(nand.spec().layout.oob_size, Some(64));
}

#[test]
fn test_nand_layout_calculations() {
    let spec = create_test_nand_spec();

    // Test pages_per_block calculation
    let pages_per_block = spec.layout.pages_per_block();
    assert_eq!(pages_per_block, 64); // 128KB / 2KB = 64 pages

    // Test total_pages calculation
    let total_pages = spec.layout.total_pages(spec.capacity);
    assert_eq!(total_pages, 65536); // 128MB / 2KB = 65536 pages
}

#[test]
fn test_mock_programmer_spi_read_bulk() {
    let mut mock = MockProgrammer::new();

    // Test bulk read
    let test_data = vec![0xAA; 4096];
    mock.expect_read(test_data.clone());

    let result = mock.spi_read_bulk(4096).unwrap();
    assert_eq!(result.len(), 4096);
    assert!(result.iter().all(|&b| b == 0xAA));
}

#[test]
fn test_mock_programmer_max_bulk_size() {
    let mock = MockProgrammer::new();
    // Mock programmer simulates CH341A bulk size
    assert_eq!(mock.max_bulk_transfer_size(), 4096);
}

// Note: Full NAND read tests require complex mock setup due to:
// 1. wait_ready polling (get_feature commands)
// 2. ECC status checking
// 3. Bad block detection (reading OOB)
//
// These would be better suited for integration tests with actual hardware
// or a more sophisticated mock framework.
