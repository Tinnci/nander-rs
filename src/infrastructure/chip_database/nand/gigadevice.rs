//! GigaDevice SPI NAND Flash Chips
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
        // 1Gbit (128MB) Series
        // =========================================================================
        nand_chip("GD5F1GM7UE", [0xC8, 0x91, 0x00], 1, 2048, 64, 128),
        nand_chip("GD5F1GQ4UA", [0xC8, 0xF1, 0x00], 1, 2048, 64, 128),
        nand_chip("GD5F1GQ4UB", [0xC8, 0xD1, 0x00], 1, 2048, 128, 128),
        nand_chip("GD5F1GQ4UC", [0xC8, 0xB1, 0x00], 1, 2048, 128, 128),
        nand_chip("GD5F1GQ4UE", [0xC8, 0xD3, 0x00], 1, 2048, 128, 128),
        nand_chip("GD5F1GQ5UE", [0xC8, 0x51, 0x00], 1, 2048, 128, 128),
        nand_chip("GD5F1GQ5RE", [0xC8, 0x41, 0x00], 1, 2048, 128, 128),
        // =========================================================================
        // 2Gbit (256MB) Series
        // =========================================================================
        nand_chip("GD5F2GQ4UB", [0xC8, 0xD2, 0x00], 2, 2048, 128, 128),
        nand_chip("GD5F2GQ4UC", [0xC8, 0xB2, 0x00], 2, 2048, 128, 128),
        nand_chip("GD5F2GQ4UE", [0xC8, 0xD5, 0x00], 2, 2048, 128, 128),
        // =========================================================================
        // 4Gbit (512MB) Series
        // =========================================================================
        nand_chip("GD5F4GQ4UB", [0xC8, 0xD4, 0x00], 4, 4096, 256, 256),
        nand_chip("GD5F4GQ4UC", [0xC8, 0xB4, 0x00], 4, 4096, 256, 256),
    ]
}

/// Helper function to create a NAND chip spec with common defaults
fn nand_chip(
    name: &str,
    jedec_id: [u8; 3],
    capacity_gbit: u32,
    page_size: u32,
    oob_size: u32,
    block_size_kb: u32,
) -> ChipSpec {
    ChipSpec {
        name: name.to_string(),
        manufacturer: MANUFACTURER_NAME.to_string(),
        jedec_id: JedecId::new(jedec_id),
        flash_type: FlashType::Nand,
        capacity: Capacity::gigabits(capacity_gbit),
        layout: ChipLayout {
            page_size,
            block_size: block_size_kb * 1024,
            oob_size: Some(oob_size),
            is_dataflash: false,
        },
        capabilities: ChipCapabilities {
            supports_ecc_control: true,
            supports_dual_spi: true,
            ..Default::default()
        },
        otp: None,
    }
}
