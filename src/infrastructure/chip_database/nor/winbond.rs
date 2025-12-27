//! Winbond SPI NOR Flash Chips
//!
//! Winbond Electronics Corporation - Manufacturer ID: 0xEF

use crate::domain::chip::*;
use crate::domain::types::*;

/// Winbond Manufacturer ID
pub const MANUFACTURER_ID: u8 = 0xEF;
pub const MANUFACTURER_NAME: &str = "Winbond";

pub fn get_chips() -> Vec<ChipSpec> {
    vec![
        // =========================================================================
        // W25X Series - Standard SPI NOR Flash
        // =========================================================================
        nor_chip("W25X05", 0x3010, 1, 64),    // 512Kbit
        nor_chip("W25X10", 0x3011, 2, 64),    // 1Mbit
        nor_chip("W25X20", 0x3012, 4, 64),    // 2Mbit
        nor_chip("W25X40", 0x3013, 8, 64),    // 4Mbit
        nor_chip("W25X80", 0x3014, 16, 64),   // 8Mbit
        nor_chip("W25X16", 0x3015, 32, 64),   // 16Mbit
        nor_chip("W25X32VS", 0x3016, 64, 64), // 32Mbit
        nor_chip("W25X64", 0x3017, 128, 64),  // 64Mbit
        // =========================================================================
        // W25Q Series - Quad SPI NOR Flash
        // =========================================================================
        nor_chip("W25Q20CL", 0x4012, 4, 64), // 2Mbit
        nor_chip("W25Q20BW", 0x5012, 4, 64),
        nor_chip("W25Q20EW", 0x6012, 4, 64),
        nor_chip("W25Q80", 0x5014, 16, 64), // 8Mbit
        nor_chip("W25Q80BL", 0x4014, 16, 64),
        nor_chip("W25Q16JQ", 0x4015, 32, 64), // 16Mbit
        nor_chip("W25Q16JM", 0x7015, 32, 64),
        nor_chip("W25Q32BV", 0x4016, 64, 64), // 32Mbit
        nor_chip("W25Q32DW", 0x6016, 64, 64),
        nor_chip("W25Q64BV", 0x4017, 128, 64), // 64Mbit
        nor_chip("W25Q64DW", 0x6017, 128, 64),
        nor_chip("W25Q128BV", 0x4018, 256, 64), // 128Mbit = 16MB
        nor_chip("W25Q128FW", 0x6018, 256, 64),
        nor_chip_4b("W25Q256FV", 0x4019, 512, 64), // 256Mbit = 32MB, 4-byte
        nor_chip_4b("W25Q512JV", 0x7119, 1024, 64), // 512Mbit = 64MB, 4-byte
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
            supports_quad_spi: true,
            ..Default::default()
        },
        otp: Some(OtpLayout {
            region_count: 3,
            region_size: 256,
            enter_opcode: 0x48, // Use as Read command for now
            exit_opcode: 0x00,
        }),
    }
}

/// Helper for 4-byte address chips (>16MB)
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
            supports_quad_spi: true,
            ..Default::default()
        },
        otp: Some(OtpLayout {
            region_count: 3,
            region_size: 256,
            enter_opcode: 0x48,
            exit_opcode: 0x00,
        }),
    }
}
