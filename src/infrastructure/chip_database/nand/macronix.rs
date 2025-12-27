//! Macronix SPI NAND Flash Chips
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
        // MX35LF Series - SPI NAND Flash
        // =========================================================================
        // 1Gbit (128MB) - MX35LF1GE4AB
        nand_chip("MX35LF1GE4AB", [0xC2, 0x12, 0x00], 1, 2048, 64, 128),
        // 2Gbit (256MB) - MX35LF2GE4AB (plane select)
        nand_chip("MX35LF2GE4AB", [0xC2, 0x22, 0x00], 2, 2048, 64, 128),
        // 2Gbit (256MB) - MX35LF2GE4AD (2-byte device ID, plane select)
        nand_chip_2id("MX35LF2GE4AD", [0xC2, 0x26, 0x03], 2, 2048, 128, 128),
        // 4Gbit (512MB) - MX35LF4GE4AD
        nand_chip("MX35LF4GE4AD", [0xC2, 0x32, 0x00], 4, 4096, 128, 256),
    ]
}

/// Helper function to create a Macronix NAND chip spec
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
        },
        capabilities: ChipCapabilities {
            supports_ecc_control: true,
            supports_dual_spi: true,
            ..Default::default()
        },
    }
}

/// Helper for chips with 2-byte device ID
fn nand_chip_2id(
    name: &str,
    jedec_id: [u8; 3],
    capacity_gbit: u32,
    page_size: u32,
    oob_size: u32,
    block_size_kb: u32,
) -> ChipSpec {
    nand_chip(
        name,
        jedec_id,
        capacity_gbit,
        page_size,
        oob_size,
        block_size_kb,
    )
}
