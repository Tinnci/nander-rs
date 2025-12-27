//! FORESEE (Longsys) SPI NAND Flash Chips
//!
//! FORESEE/Longsys - Manufacturer ID: 0xCD

use crate::domain::chip::*;
use crate::domain::types::*;

/// FORESEE Manufacturer ID
pub const MANUFACTURER_ID: u8 = 0xCD;
pub const MANUFACTURER_NAME: &str = "FORESEE";

pub fn get_chips() -> Vec<ChipSpec> {
    vec![
        // =========================================================================
        // FS35ND Series - SPI NAND Flash
        // =========================================================================
        // 1Gbit (128MB)
        nand_chip("FS35ND01GD1F1", [0xCD, 0xA1, 0x00], 1, 2048, 64, 128),
        nand_chip("FS35ND01GS1F1", [0xCD, 0xB1, 0x00], 1, 2048, 128, 128),
        // 2Gbit (256MB)
        nand_chip("FS35ND02GS2F1", [0xCD, 0xA2, 0x00], 2, 2048, 64, 128),
        nand_chip("FS35ND02GD1F1", [0xCD, 0xB2, 0x00], 2, 2048, 128, 128),
        // 1Gbit alternative (F35SQA series)
        nand_chip("F35SQA001G", [0xCD, 0x71, 0x00], 1, 2048, 64, 128),
    ]
}

/// Helper function to create a FORESEE NAND chip spec
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
