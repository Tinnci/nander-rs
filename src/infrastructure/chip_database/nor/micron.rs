//! Micron/ST SPI NOR Flash Chips
//!
//! Micron/STMicroelectronics - Manufacturer ID: 0x20

use crate::domain::chip::*;
use crate::domain::types::*;

/// Micron/ST Manufacturer ID
pub const MANUFACTURER_ID: u8 = 0x20;
pub const MANUFACTURER_NAME: &str = "Micron";

pub fn get_chips() -> Vec<ChipSpec> {
    vec![
        // =========================================================================
        // M25P/N25Q Series - Legacy SPI NOR Flash
        // =========================================================================
        nor_chip("M25P016", 0x2015, 32, 64),   // 16Mbit
        nor_chip("N25Q032A", 0xBA16, 64, 64),  // 32Mbit
        nor_chip("N25Q064A", 0xBA17, 128, 64), // 64Mbit
        nor_chip("M25P128", 0x2018, 256, 64),  // 128Mbit
        nor_chip("N25Q128A", 0xBA18, 256, 64),
        nor_chip_4b("N25Q256A", 0xBA19, 512, 64), // 256Mbit, 4-byte
        nor_chip_4b("MT25QL512AB", 0xBA20, 1024, 64), // 512Mbit, 4-byte
        // =========================================================================
        // XM25QH Series (XMC Corporation, similar ID range)
        // =========================================================================
        nor_chip("XM25QH32B", 0x4016, 64, 64),
        nor_chip("XM25QH32A", 0x7016, 64, 64),
        nor_chip("XM25QH64A", 0x7017, 128, 64),
        nor_chip("XM25QH64C", 0x4017, 128, 64),
        nor_chip("XM25QH128A", 0x7018, 256, 64),
        nor_chip("XM25QH128C", 0x4018, 256, 64),
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
        otp: None,
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
        otp: None,
    }
}
