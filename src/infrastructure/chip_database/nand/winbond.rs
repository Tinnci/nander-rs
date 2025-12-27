//! Winbond SPI NAND Flash Chips
//!
//! Winbond Electronics Corporation - Manufacturer ID: 0xEF

use crate::domain::chip::*;
use crate::domain::types::*;

/// Winbond Manufacturer ID
pub const MANUFACTURER_ID: u8 = 0xEF;
pub const MANUFACTURER_NAME: &str = "Winbond";

pub fn get_chips() -> Vec<ChipSpec> {
    vec![
        // =========================================================================
        // W25N Series - SPI NAND Flash
        // =========================================================================
        // 1Gbit (128MB)
        nand_chip_2id("W25N01GV", [0xEF, 0xAA, 0x21], 1, 2048, 64, 128),
        // 2Gbit (256MB)
        nand_chip_2id("W25N02KV", [0xEF, 0xAA, 0x22], 2, 2048, 128, 128),
        // 4Gbit (512MB)
        nand_chip_2id("W25N04KV", [0xEF, 0xAA, 0x23], 4, 2048, 128, 128),
        // =========================================================================
        // W25M Series - Multi-die NAND
        // =========================================================================
        // 2Gbit (256MB) - Dual Die
        nand_chip_2id("W25M02GV", [0xEF, 0xAB, 0x21], 2, 2048, 64, 128),
    ]
}

/// Helper function to create a NAND chip spec (Winbond uses 2-byte device ID)
fn nand_chip_2id(
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
