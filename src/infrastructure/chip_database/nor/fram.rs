//! SPI FRAM (Ferroelectric RAM) Chip Database
//!
//! FRAM chips are non-volatile like Flash but:
//! - No erase required before write
//! - High endurance (10^14 cycles)
//! - Fast write (no wait time)

use crate::domain::chip::*;
use crate::domain::types::*;

pub fn get_chips() -> Vec<ChipSpec> {
    vec![
        // =========================================================================
        // Cypress/Infineon Excelon FRAM
        // =========================================================================
        fram_chip("CY15B104Q", 0x0426, 512), // 4Mbit = 512KB
        fram_chip("CY15B102Q", 0x0425, 256), // 2Mbit = 256KB
        fram_chip("FM25V02", 0x7F01, 32),    // 256Kbit = 32KB
        fram_chip("FM25V01", 0x7F00, 16),    // 128Kbit = 16KB
    ]
}

fn fram_chip(name: &str, jedec_id: u16, size_kb: u32) -> ChipSpec {
    ChipSpec {
        name: name.to_string(),
        manufacturer: "Cypress".to_string(),
        jedec_id: JedecId::new([0x04, (jedec_id >> 8) as u8, jedec_id as u8]),
        flash_type: FlashType::SpiFram,
        capacity: Capacity::bytes(size_kb * 1024),
        layout: ChipLayout {
            page_size: 256, // FRAM doesn't really have pages, but we use 256 for compatibility
            block_size: 256,
            oob_size: None,
            is_dataflash: false,
        },
        capabilities: ChipCapabilities::default(),
        otp: None,
    }
}
