//! Other Manufacturers SPI NAND Flash Chips
//!
//! This module contains chips from smaller manufacturers:
//! - HEYANG (0xC9)
//! - PN (Paragon/Zbit) (0xA1)
//! - ATO (0x9B, 0xAD)
//! - FM (Fudan Microelectronics) (0xA1)
//! - DS (Dosilicon) (0xE5)
//! - BIWIN (0xBC)
//! - Zentel (0xC8)

use crate::domain::chip::*;
use crate::domain::types::*;

pub fn get_chips() -> Vec<ChipSpec> {
    let mut chips = Vec::new();
    chips.extend(heyang_chips());
    chips.extend(pn_chips());
    chips.extend(ato_chips());
    chips.extend(fm_chips());
    chips.extend(ds_chips());
    chips.extend(zentel_chips());
    chips
}

// =========================================================================
// HEYANG Chips (0xC9)
// =========================================================================
fn heyang_chips() -> Vec<ChipSpec> {
    vec![
        nand_chip(
            "HEYANG",
            "HYF1GQ4UAACAE",
            [0xC9, 0x51, 0x00],
            1,
            2048,
            128,
            128,
        ),
        nand_chip(
            "HEYANG",
            "HYF2GQ4UAACAE",
            [0xC9, 0x52, 0x00],
            2,
            2048,
            128,
            128,
        ),
        nand_chip(
            "HEYANG",
            "HYF2GQ4UHCCAE",
            [0xC9, 0x5A, 0x00],
            2,
            2048,
            128,
            128,
        ),
        nand_chip(
            "HEYANG",
            "HYF1GQ4UDACAE",
            [0xC9, 0x21, 0x00],
            1,
            2048,
            64,
            128,
        ),
        nand_chip(
            "HEYANG",
            "HYF2GQ4UDACAE",
            [0xC9, 0x22, 0x00],
            2,
            2048,
            64,
            128,
        ),
    ]
}

// =========================================================================
// PN/Paragon Chips (0xA1)
// =========================================================================
fn pn_chips() -> Vec<ChipSpec> {
    vec![
        nand_chip("PN", "PN26G01A-X", [0xA1, 0xE1, 0x00], 1, 2048, 128, 128),
        nand_chip("PN", "PN26G02A-X", [0xA1, 0xE2, 0x00], 2, 2048, 128, 128),
        nand_chip("PN", "PN26Q01A-X", [0xA1, 0xC1, 0x00], 1, 2048, 128, 128),
    ]
}

// =========================================================================
// ATO Chips (0x9B, 0xAD)
// =========================================================================
fn ato_chips() -> Vec<ChipSpec> {
    vec![
        nand_chip("ATO", "ATO25D1GA", [0x9B, 0x12, 0x00], 1, 2048, 64, 128),
        nand_chip("ATO", "ATO25D2GA", [0x9B, 0xF1, 0x00], 2, 2048, 64, 128),
        nand_chip("ATO", "ATO25D2GB", [0xAD, 0xDA, 0x00], 2, 2048, 128, 128),
    ]
}

// =========================================================================
// FM (Fudan Microelectronics) Chips (0xA1)
// =========================================================================
fn fm_chips() -> Vec<ChipSpec> {
    vec![
        nand_chip("FM", "FM25S01", [0xA1, 0xA1, 0x00], 1, 2048, 128, 128),
        nand_chip("FM", "FM25S01A", [0xA1, 0xE4, 0x00], 1, 2048, 64, 128),
        nand_chip("FM", "FM25G01B", [0xA1, 0xD1, 0x00], 1, 2048, 128, 128),
        nand_chip("FM", "FM25G02B", [0xA1, 0xD2, 0x00], 2, 2048, 128, 128),
        nand_chip("FM", "FM25G02", [0xA1, 0xF2, 0x00], 2, 2048, 64, 128),
        nand_chip("FM", "FM25G02C", [0xA1, 0x92, 0x00], 2, 2048, 64, 128),
    ]
}

// =========================================================================
// DS (Dosilicon) Chips (0xE5)
// =========================================================================
fn ds_chips() -> Vec<ChipSpec> {
    vec![
        nand_chip(
            "Dosilicon",
            "DS35Q1GA",
            [0xE5, 0x71, 0x00],
            1,
            2048,
            64,
            128,
        ),
        nand_chip(
            "Dosilicon",
            "DS35Q2GA",
            [0xE5, 0x72, 0x00],
            2,
            2048,
            64,
            128,
        ),
        nand_chip(
            "Dosilicon",
            "DS35Q1GB",
            [0xE5, 0xF1, 0x00],
            1,
            2048,
            128,
            128,
        ),
        nand_chip(
            "Dosilicon",
            "DS35Q2GB",
            [0xE5, 0xF2, 0x00],
            2,
            2048,
            128,
            128,
        ),
    ]
}

// =========================================================================
// Zentel Chips (0xC8 - shares with GigaDevice)
// =========================================================================
fn zentel_chips() -> Vec<ChipSpec> {
    vec![
        nand_chip_custom(
            "Zentel",
            "A5U12A21ASC",
            [0xC8, 0x20, 0x00],
            64,
            2048,
            64,
            128,
        ),
        nand_chip(
            "Zentel",
            "A5U1GA21BWS",
            [0xC8, 0x21, 0x00],
            1,
            2048,
            64,
            128,
        ),
    ]
}

/// Helper function to create a NAND chip spec (generic manufacturer)
fn nand_chip(
    manufacturer: &str,
    name: &str,
    jedec_id: [u8; 3],
    capacity_gbit: u32,
    page_size: u32,
    oob_size: u32,
    block_size_kb: u32,
) -> ChipSpec {
    ChipSpec {
        name: name.to_string(),
        manufacturer: manufacturer.to_string(),
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

/// For chips with custom capacity (not standard Gbit sizes)
fn nand_chip_custom(
    manufacturer: &str,
    name: &str,
    jedec_id: [u8; 3],
    capacity_mb: u32,
    page_size: u32,
    oob_size: u32,
    block_size_kb: u32,
) -> ChipSpec {
    ChipSpec {
        name: name.to_string(),
        manufacturer: manufacturer.to_string(),
        jedec_id: JedecId::new(jedec_id),
        flash_type: FlashType::Nand,
        capacity: Capacity::megabytes(capacity_mb),
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
