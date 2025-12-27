//! Micron SPI NAND Flash Chips
//!
//! Micron Technology, Inc. - Manufacturer ID: 0x2C

use crate::domain::chip::*;
use crate::domain::types::*;

/// Micron Manufacturer ID
pub const MANUFACTURER_ID: u8 = 0x2C;
pub const MANUFACTURER_NAME: &str = "Micron";

pub fn get_chips() -> Vec<ChipSpec> {
    vec![
        // =========================================================================
        // MT29F Series - SPI NAND Flash
        // =========================================================================
        // 1Gbit (128MB)
        nand_chip("MT29F1G01", [0x2C, 0x14, 0x00], 1, 2048, 128, 128),
        // 2Gbit (256MB) - Has plane select
        nand_chip("MT29F2G01", [0x2C, 0x24, 0x00], 2, 2048, 128, 128),
        // 4Gbit (512MB) - Has plane select and die select
        nand_chip("MT29F4G01", [0x2C, 0x36, 0x00], 4, 2048, 128, 128),
    ]
}

/// Helper function to create a Micron NAND chip spec
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
