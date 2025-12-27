//! I2C EEPROM (24Cxx Series) Chip Database
//!
//! This module defines supported I2C EEPROM chips.

use crate::domain::chip::*;
use crate::domain::types::*;

/// Get all I2C EEPROM chip definitions
pub fn get_all_i2c_eeprom() -> Vec<ChipSpec> {
    vec![
        i2c_eeprom("24C01", 128, 8),        // 1Kbit = 128 bytes, 8-byte page
        i2c_eeprom("24C02", 256, 8),        // 2Kbit = 256 bytes, 8-byte page
        i2c_eeprom("24C04", 512, 16),       // 4Kbit = 512 bytes, 16-byte page
        i2c_eeprom("24C08", 1024, 16),      // 8Kbit = 1KB, 16-byte page
        i2c_eeprom("24C16", 2048, 16),      // 16Kbit = 2KB, 16-byte page
        i2c_eeprom("24C32", 4096, 32),      // 32Kbit = 4KB, 32-byte page
        i2c_eeprom("24C64", 8192, 32),      // 64Kbit = 8KB, 32-byte page
        i2c_eeprom("24C128", 16384, 64),    // 128Kbit = 16KB, 64-byte page
        i2c_eeprom("24C256", 32768, 64),    // 256Kbit = 32KB, 64-byte page
        i2c_eeprom("24C512", 65536, 128),   // 512Kbit = 64KB, 128-byte page
        i2c_eeprom("24C1024", 131072, 256), // 1024Kbit = 128KB, 256-byte page
    ]
}

/// Helper to create an I2C EEPROM spec
fn i2c_eeprom(name: &str, capacity_bytes: u32, page_size: u32) -> ChipSpec {
    // Synthetic ID for I2C EEPROM
    // 0xFD marker | capacity_code | page_code
    let capacity_code = match capacity_bytes {
        128 => 0x01,
        256 => 0x02,
        512 => 0x03,
        1024 => 0x04,
        2048 => 0x05,
        4096 => 0x06,
        8192 => 0x07,
        16384 => 0x08,
        32768 => 0x09,
        65536 => 0x0A,
        131072 => 0x0B,
        _ => 0x00,
    };

    ChipSpec {
        name: name.to_string(),
        manufacturer: "Generic I2C EEPROM".to_string(),
        jedec_id: JedecId::new([0xFD, capacity_code, 0x00]),
        flash_type: FlashType::I2cEeprom,
        capacity: Capacity::bytes(capacity_bytes),
        layout: ChipLayout {
            page_size,
            block_size: page_size,
            oob_size: None,
        },
        capabilities: ChipCapabilities::default(),
    }
}
