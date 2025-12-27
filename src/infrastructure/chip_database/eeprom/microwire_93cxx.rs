//! Microwire EEPROM (93Cxx Series) Chip Database
//!
//! This module defines supported Microwire EEPROM chips.

use crate::domain::chip::*;
use crate::domain::types::*;

/// Get all Microwire EEPROM chip definitions
pub fn get_all_microwire_eeprom() -> Vec<ChipSpec> {
    vec![
        mw_eeprom("93C06", 32),   // 256 bit = 32 bytes
        mw_eeprom("93C46", 128),  // 1 Kbit = 128 bytes
        mw_eeprom("93C56", 256),  // 2 Kbit = 256 bytes
        mw_eeprom("93C66", 512),  // 4 Kbit = 512 bytes
        mw_eeprom("93C76", 1024), // 8 Kbit = 1 KB
        mw_eeprom("93C86", 2048), // 16 Kbit = 2 KB
    ]
}

/// Helper to create a Microwire EEPROM spec
fn mw_eeprom(name: &str, capacity_bytes: u32) -> ChipSpec {
    // Synthetic ID for Microwire EEPROM
    // 0xFC marker | capacity_code | 0x00
    let capacity_code = match capacity_bytes {
        32 => 0x01,
        128 => 0x02,
        256 => 0x03,
        512 => 0x04,
        1024 => 0x05,
        2048 => 0x06,
        _ => 0x00,
    };

    ChipSpec {
        name: name.to_string(),
        manufacturer: "Generic Microwire EEPROM".to_string(),
        jedec_id: JedecId::new([0xFC, capacity_code, 0x00]),
        flash_type: FlashType::MicrowireEeprom,
        capacity: Capacity::bytes(capacity_bytes),
        layout: ChipLayout {
            page_size: 1, // Microwire is byte-writable
            block_size: 1,
            oob_size: None,
            is_dataflash: false,
        },
        capabilities: ChipCapabilities::default(),
        otp: None,
    }
}
