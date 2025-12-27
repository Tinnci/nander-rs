//! Etron Technology SPI NAND Flash Chips
//!
//! Etron Technology, Inc. - Manufacturer ID: 0xD5

use crate::domain::chip::*;
use crate::domain::types::*;

/// Etron Manufacturer ID
pub const MANUFACTURER_ID: u8 = 0xD5;
pub const MANUFACTURER_NAME: &str = "Etron";

pub fn get_chips() -> Vec<ChipSpec> {
    vec![
        // =========================================================================
        // EM73C Series - 1Gbit (128MB)
        // =========================================================================
        nand_chip("EM73C044SNB", [0xD5, 0x11, 0x00], 1, 2048, 128, 128),
        nand_chip("EM73C044SND", [0xD5, 0x1D, 0x00], 1, 2048, 64, 128),
        nand_chip("EM73C044SNF", [0xD5, 0x09, 0x00], 1, 2048, 128, 128),
        nand_chip("EM73C044VCA", [0xD5, 0x18, 0x00], 1, 2048, 64, 128),
        nand_chip("EM73C044VCD", [0xD5, 0x1C, 0x00], 1, 2048, 64, 128),
        // =========================================================================
        // EM73D Series - 2Gbit (256MB)
        // =========================================================================
        nand_chip("EM73D044SNA", [0xD5, 0x12, 0x00], 2, 2048, 128, 128),
        nand_chip("EM73D044SNC", [0xD5, 0x0A, 0x00], 2, 2048, 128, 128),
        nand_chip("EM73D044SND", [0xD5, 0x1E, 0x00], 2, 2048, 64, 128),
        nand_chip("EM73D044SNF", [0xD5, 0x10, 0x00], 2, 2048, 128, 128),
        nand_chip("EM73D044VCA", [0xD5, 0x13, 0x00], 2, 2048, 128, 128),
        nand_chip("EM73D044VCB", [0xD5, 0x14, 0x00], 2, 2048, 64, 128),
        nand_chip("EM73D044VCD", [0xD5, 0x17, 0x00], 2, 2048, 128, 128),
        nand_chip("EM73D044VCG", [0xD5, 0x1F, 0x00], 2, 2048, 64, 128),
        nand_chip("EM73D044VCH", [0xD5, 0x1B, 0x00], 2, 2048, 64, 128),
        // =========================================================================
        // EM73E Series - 4Gbit (512MB)
        // =========================================================================
        nand_chip("EM73E044SNA", [0xD5, 0x03, 0x00], 4, 4096, 256, 256),
    ]
}

/// Helper function to create an Etron NAND chip spec
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
