//! GigaDevice SPI NOR Flash Chips
//!
//! GigaDevice Corporation - Manufacturer ID: 0xC8

use crate::domain::chip::*;
use crate::domain::types::*;

/// GigaDevice Manufacturer ID
pub const MANUFACTURER_ID: u8 = 0xC8;
pub const MANUFACTURER_NAME: &str = "GigaDevice";

pub fn get_chips() -> Vec<ChipSpec> {
    vec![
        // =========================================================================
        // GD25Q Series - Standard SPI NOR Flash
        // =========================================================================
        nor_chip("GD25Q16", 0x4015, 32, 64),       // 16Mbit = 2MB
        nor_chip("GD25Q32", 0x4016, 64, 64),       // 32Mbit = 4MB
        nor_chip("GD25Q64CSIG", 0x4017, 128, 64),  // 64Mbit = 8MB
        nor_chip("GD25Q128CSIG", 0x4018, 256, 64), // 128Mbit = 16MB
        nor_chip_4b("GD25Q256CSIG", 0x4019, 512, 64), // 256Mbit = 32MB, 4-byte addr
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
            page_size: 256, // Standard NOR page size
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

/// Helper function to create a NOR chip spec (4-byte address for >16MB)
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
