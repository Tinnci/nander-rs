//! EON (ESMT) SPI NOR Flash Chips
//!
//! EON Silicon Solution - Manufacturer ID: 0x1C

use crate::domain::chip::*;
use crate::domain::types::*;

/// EON Manufacturer ID
pub const MANUFACTURER_ID: u8 = 0x1C;
pub const MANUFACTURER_NAME: &str = "EON";

pub fn get_chips() -> Vec<ChipSpec> {
    vec![
        // =========================================================================
        // EN25F/Q/QH Series - SPI NOR Flash
        // =========================================================================
        nor_chip("EN25F16", 0x3115, 32, 64),
        nor_chip("EN25Q16", 0x3015, 32, 64),
        nor_chip("EN25QH16", 0x7015, 32, 64),
        nor_chip("EN25Q32B", 0x3016, 64, 64),
        nor_chip("EN25F32", 0x3116, 64, 64),
        nor_chip("EN25F64", 0x2017, 128, 64),
        nor_chip("EN25Q64", 0x3017, 128, 64),
        nor_chip("EN25QA64A", 0x6017, 128, 64),
        nor_chip("EN25QH64A", 0x7017, 128, 64),
        nor_chip("EN25Q128", 0x3018, 256, 64),
        nor_chip("EN25QA128A", 0x6018, 256, 64),
        nor_chip("EN25QH128A", 0x7018, 256, 64),
        nor_chip("GM25Q128A", 0x4018, 256, 64),
        nor_chip_4b("EN25Q256", 0x7019, 512, 64),
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
            ..Default::default()
        },
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
        },
        capabilities: ChipCapabilities {
            supports_4byte_addr: true,
            ..Default::default()
        },
    }
}
