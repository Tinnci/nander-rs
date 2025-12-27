//! Macronix SPI NOR Flash Chips
//!
//! Macronix International Co., Ltd. - Manufacturer ID: 0xC2

use crate::domain::chip::*;
use crate::domain::types::*;

/// Macronix Manufacturer ID
pub const MANUFACTURER_ID: u8 = 0xC2;
pub const MANUFACTURER_NAME: &str = "Macronix";

pub fn get_chips() -> Vec<ChipSpec> {
    vec![
        // =========================================================================
        // MX25L Series - SPI NOR Flash
        // =========================================================================
        nor_chip("MX25L4005A", 0x2013, 8, 64),  // 4Mbit = 512KB
        nor_chip("MX25L8005M", 0x2014, 16, 64), // 8Mbit = 1MB
        nor_chip("MX25L1605D", 0x2015, 32, 64), // 16Mbit = 2MB
        nor_chip("MX25L3205D", 0x2016, 64, 64), // 32Mbit = 4MB
        nor_chip("MX25L6405D", 0x2017, 128, 64), // 64Mbit = 8MB
        nor_chip("MX25L12805D", 0x2018, 256, 64), // 128Mbit = 16MB
        nor_chip_4b("MX25L25635E", 0x2019, 512, 64), // 256Mbit = 32MB
        nor_chip_4b("MX25L51245G", 0x201a, 1024, 64), // 512Mbit = 64MB
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
        },
        capabilities: ChipCapabilities {
            supports_4byte_addr: false,
            supports_quad_spi: true,
            ..Default::default()
        },
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
        },
        capabilities: ChipCapabilities {
            supports_4byte_addr: true,
            supports_quad_spi: true,
            ..Default::default()
        },
    }
}
