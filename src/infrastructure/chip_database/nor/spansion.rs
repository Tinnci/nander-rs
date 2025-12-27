//! Spansion/Cypress SPI NOR Flash Chips
//!
//! Spansion (now Infineon) - Manufacturer ID: 0x01

use crate::domain::chip::*;
use crate::domain::types::*;

/// Spansion Manufacturer ID
pub const MANUFACTURER_ID: u8 = 0x01;
pub const MANUFACTURER_NAME: &str = "Spansion";

pub fn get_chips() -> Vec<ChipSpec> {
    vec![
        // =========================================================================
        // FL Series - Legacy SPI NOR Flash
        // =========================================================================
        nor_chip("FL016AIF", 0x0214, 32, 64),  // 16Mbit = 2MB
        nor_chip("FL064AIF", 0x0216, 128, 64), // 64Mbit = 8MB
        // =========================================================================
        // S25FL Series - Modern SPI NOR Flash
        // =========================================================================
        nor_chip("S25FL016P", 0x0214, 32, 64),
        nor_chip("S25FL032P", 0x0215, 64, 64),
        nor_chip("S25FL064P", 0x0216, 128, 64),
        nor_chip("S25FL128P", 0x2018, 256, 64),
        nor_chip("S25FL129P", 0x2018, 256, 64),
        nor_chip_4b("S25FL256S", 0x0219, 512, 64), // 4-byte address
        // =========================================================================
        // S25FL1xxK Series - Uniform Sector
        // =========================================================================
        nor_chip("S25FL116K", 0x4015, 32, 64),
        nor_chip("S25FL132K", 0x4016, 64, 64),
        nor_chip("S25FL164K", 0x4017, 128, 64),
    ]
}

/// Helper function to create a NOR chip spec (3-byte address)
fn nor_chip(name: &str, jedec_id: u16, n_sectors: u32, sector_size_kb: u32) -> ChipSpec {
    let capacity_bytes = n_sectors * sector_size_kb * 1024;

    ChipSpec {
        name: name.to_string(),
        manufacturer: MANUFACTURER_NAME.to_string(),
        jedec_id: JedecId::new([MANUFACTURER_ID, (jedec_id >> 8) as u8, jedec_id as u8]),
        flash_type: FlashType::Nor,
        capacity: Capacity::bytes(capacity_bytes),
        layout: ChipLayout {
            page_size: 256,
            block_size: sector_size_kb * 1024,
            oob_size: None,
            is_dataflash: false,
        },
        capabilities: ChipCapabilities {
            supports_4byte_addr: false,
            ..Default::default()
        },
        otp: Some(OtpLayout {
            region_count: 1,
            region_size: 1024,
            enter_opcode: 0x4B,
            exit_opcode: 0x00,
        }),
    }
}

/// Helper for 4-byte address chips
fn nor_chip_4b(name: &str, jedec_id: u16, n_sectors: u32, sector_size_kb: u32) -> ChipSpec {
    let capacity_bytes = n_sectors * sector_size_kb * 1024;

    ChipSpec {
        name: name.to_string(),
        manufacturer: MANUFACTURER_NAME.to_string(),
        jedec_id: JedecId::new([MANUFACTURER_ID, (jedec_id >> 8) as u8, jedec_id as u8]),
        flash_type: FlashType::Nor,
        capacity: Capacity::bytes(capacity_bytes),
        layout: ChipLayout {
            page_size: 256,
            block_size: sector_size_kb * 1024,
            oob_size: None,
            is_dataflash: false,
        },
        capabilities: ChipCapabilities {
            supports_4byte_addr: true,
            ..Default::default()
        },
        otp: Some(OtpLayout {
            region_count: 1,
            region_size: 1024,
            enter_opcode: 0x4B,
            exit_opcode: 0x00,
        }),
    }
}
