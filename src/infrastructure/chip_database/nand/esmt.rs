//! ESMT (Elite Semiconductor) SPI NAND Flash Chips
//!
//! ESMT/Zentel - Manufacturer ID: 0xC8 (shares with GigaDevice)

use crate::domain::chip::*;
use crate::domain::types::*;

/// ESMT uses same ID as GigaDevice in some cases
pub const MANUFACTURER_ID: u8 = 0xC8;
pub const MANUFACTURER_NAME: &str = "ESMT";

pub fn get_chips() -> Vec<ChipSpec> {
    vec![
        // =========================================================================
        // F50 Series - SPI NAND Flash
        // =========================================================================
        // 512Mbit (64MB)
        nand_chip("F50L512M41A", [0xC8, 0x20, 0x00], 0, 2048, 64, 128, 64),
        // 1Gbit (128MB)
        nand_chip("F50L1G41A0", [0xC8, 0x21, 0x00], 1, 2048, 64, 128, 128),
        nand_chip("F50L1G41LB", [0xC8, 0x01, 0x00], 1, 2048, 64, 128, 128),
        nand_chip("F50D1G41LB", [0xC8, 0x11, 0x00], 1, 2048, 128, 128, 128),
        // 2Gbit (256MB)
        nand_chip("F50L2G41LB", [0xC8, 0x0A, 0x00], 2, 2048, 64, 128, 256),
    ]
}

/// Helper function to create an ESMT NAND chip spec
/// For 512Mbit chips, capacity_gbit = 0a and capacity_mb is used
fn nand_chip(
    name: &str,
    jedec_id: [u8; 3],
    capacity_gbit: u32,
    page_size: u32,
    oob_size: u32,
    block_size_kb: u32,
    capacity_mb: u32,
) -> ChipSpec {
    let capacity = if capacity_gbit > 0 {
        Capacity::gigabits(capacity_gbit)
    } else {
        Capacity::megabytes(capacity_mb)
    };

    ChipSpec {
        name: name.to_string(),
        manufacturer: MANUFACTURER_NAME.to_string(),
        jedec_id: JedecId::new(jedec_id),
        flash_type: FlashType::Nand,
        capacity,
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
