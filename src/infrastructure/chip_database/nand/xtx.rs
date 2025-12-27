//! XTX Technology SPI NAND Flash Chips
//!
//! XTX Technology Limited - Manufacturer ID: 0x0B

use crate::domain::chip::*;
use crate::domain::types::*;

/// XTX Manufacturer ID
pub const MANUFACTURER_ID: u8 = 0x0B;
pub const MANUFACTURER_NAME: &str = "XTX";

pub fn get_chips() -> Vec<ChipSpec> {
    vec![
        // =========================================================================
        // XT26G Series - SPI NAND Flash
        // =========================================================================
        // 1Gbit (128MB)
        nand_chip("XT26G01A", [0x0B, 0xE1, 0x00], 1, 2048, 64, 128),
        nand_chip("XT26G01C", [0x0B, 0x11, 0x00], 1, 2048, 128, 128),
        // 2Gbit (256MB)
        nand_chip("XT26G02A", [0x0B, 0xE2, 0x00], 2, 2048, 64, 128),
        nand_chip("XT26G02B", [0x0B, 0xF2, 0x00], 2, 2048, 64, 128),
    ]
}

/// Helper function to create an XTX NAND chip spec
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
