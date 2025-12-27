//! Other Manufacturers SPI NOR Flash Chips
//!
//! This module contains chips from smaller manufacturers:
//!
//! - Atmel (0x1F)
//! - ESMT (0x8C)
//! - Zbit/ZB (0x5E)
//! - BOYA (0x68)
//! - XTX (0x0B)
//! - ISSI (0x9D)
//! - FM (0xA1, 0xF8)
//! - Zetta (0xBA)
//! - PN (0xE0)
//! - PUYA (0x85)
//!
//! And more...

use crate::domain::chip::*;
use crate::domain::types::*;

pub fn get_chips() -> Vec<ChipSpec> {
    let mut chips = Vec::new();
    chips.extend(atmel_chips());
    chips.extend(esmt_nor_chips());
    chips.extend(zbit_chips());
    chips.extend(boya_chips());
    chips.extend(xtx_nor_chips());
    chips.extend(issi_chips());
    chips.extend(fm_nor_chips());
    chips.extend(zetta_chips());
    chips.extend(puya_chips());
    chips
}

// =========================================================================
// Atmel (0x1F)
// =========================================================================
fn atmel_chips() -> Vec<ChipSpec> {
    vec![
        nor_chip_with_id("Atmel", "AT25DF321", 0x1F, 0x4700, 64, 64),
        nor_chip_with_id("Atmel", "AT26DF161", 0x1F, 0x4600, 32, 64),
    ]
}

// =========================================================================
// ESMT (0x8C) - NOR variants
// =========================================================================
fn esmt_nor_chips() -> Vec<ChipSpec> {
    vec![
        nor_chip_with_id("ESMT", "F25L016", 0x8C, 0x2115, 32, 64),
        nor_chip_with_id("ESMT", "F25L16QA", 0x8C, 0x4115, 32, 64),
        nor_chip_with_id("ESMT", "F25L032", 0x8C, 0x2116, 64, 64),
        nor_chip_with_id("ESMT", "F25L32QA", 0x8C, 0x4116, 64, 64),
        nor_chip_with_id("ESMT", "F25L064", 0x8C, 0x2117, 128, 64),
        nor_chip_with_id("ESMT", "F25L64QA", 0x8C, 0x4117, 128, 64),
    ]
}

// =========================================================================
// Zbit/ZB (0x5E)
// =========================================================================
fn zbit_chips() -> Vec<ChipSpec> {
    vec![
        nor_chip_with_id("Zbit", "ZB25VQ16", 0x5E, 0x4015, 32, 64),
        nor_chip_with_id("Zbit", "ZB25VQ32", 0x5E, 0x4016, 64, 64),
        nor_chip_with_id("Zbit", "ZB25VQ64", 0x5E, 0x4017, 128, 64),
        nor_chip_with_id("Zbit", "ZB25VQ128", 0x5E, 0x4018, 256, 64),
    ]
}

// =========================================================================
// BOYA (0x68)
// =========================================================================
fn boya_chips() -> Vec<ChipSpec> {
    vec![
        nor_chip_with_id("BOYA", "BY25Q16BS", 0x68, 0x4015, 32, 64),
        nor_chip_with_id("BOYA", "BY25Q32BS", 0x68, 0x4016, 64, 64),
        nor_chip_with_id("BOYA", "BY25Q64AS", 0x68, 0x4017, 128, 64),
        nor_chip_with_id("BOYA", "BY25Q128AS", 0x68, 0x4018, 256, 64),
    ]
}

// =========================================================================
// XTX NOR (0x0B)
// =========================================================================
fn xtx_nor_chips() -> Vec<ChipSpec> {
    vec![
        nor_chip_with_id("XTX", "XT25F32B", 0x0B, 0x4016, 64, 64),
        nor_chip_with_id("XTX", "XT25F64B", 0x0B, 0x4017, 128, 64),
        nor_chip_with_id("XTX", "XT25F128B", 0x0B, 0x4018, 256, 64),
        nor_chip_with_id("XTX", "XT25Q128D", 0x0B, 0x6018, 256, 64),
    ]
}

// =========================================================================
// ISSI (0x9D)
// =========================================================================
fn issi_chips() -> Vec<ChipSpec> {
    vec![
        nor_chip_with_id("ISSI", "IC25LP016", 0x9D, 0x6015, 32, 64),
        nor_chip_with_id("ISSI", "IC25LP032", 0x9D, 0x6016, 64, 64),
        nor_chip_with_id("ISSI", "IC25LP064", 0x9D, 0x6017, 128, 64),
        nor_chip_with_id("ISSI", "IC25LP128", 0x9D, 0x6018, 256, 64),
    ]
}

// =========================================================================
// FM (0xA1, 0xF8)
// =========================================================================
fn fm_nor_chips() -> Vec<ChipSpec> {
    vec![
        nor_chip_with_id("FM", "FS25Q016", 0xA1, 0x4015, 32, 64),
        nor_chip_with_id("FM", "FS25Q032", 0xA1, 0x4016, 64, 64),
        nor_chip_with_id("FM", "FS25Q064", 0xA1, 0x4017, 128, 64),
        nor_chip_with_id("FM", "FS25Q128", 0xA1, 0x4018, 256, 64),
        nor_chip_with_id("FM", "FM25W16", 0xA1, 0x2815, 32, 64),
        nor_chip_with_id("FM", "FM25W32", 0xA1, 0x2816, 64, 64),
        nor_chip_with_id("FM", "FM25W64", 0xA1, 0x2817, 128, 64),
        nor_chip_with_id("FM", "FM25W128", 0xA1, 0x2818, 256, 64),
        nor_chip_with_id("FM", "FM25Q16A", 0xF8, 0x3215, 32, 64),
        nor_chip_with_id("FM", "FM25Q32A", 0xF8, 0x3216, 64, 64),
        nor_chip_with_id("FM", "FM25Q64A", 0xF8, 0x3217, 128, 64),
        nor_chip_with_id("FM", "FM25Q128A", 0xF8, 0x3218, 256, 64),
    ]
}

// =========================================================================
// Zetta (0xBA)
// =========================================================================
fn zetta_chips() -> Vec<ChipSpec> {
    vec![
        nor_chip_with_id("Zetta", "ZD25Q16A", 0xBA, 0x4015, 32, 64),
        nor_chip_with_id("Zetta", "ZD25Q32A", 0xBA, 0x4016, 64, 64),
        nor_chip_with_id("Zetta", "ZD25Q64A", 0xBA, 0x4017, 128, 64),
        nor_chip_with_id("Zetta", "ZD25Q128A", 0xBA, 0x4018, 256, 64),
        nor_chip_with_id("Zetta", "ZD25Q16B", 0xBA, 0x3215, 32, 64),
        nor_chip_with_id("Zetta", "ZD25Q32B", 0xBA, 0x3216, 64, 64),
        nor_chip_with_id("Zetta", "ZD25Q64B", 0xBA, 0x3217, 128, 64),
        nor_chip_with_id("Zetta", "ZD25Q128B", 0xBA, 0x3218, 256, 64),
        nor_chip_with_id("Zetta", "ZD25Q16C", 0xBA, 0x6015, 32, 64),
        nor_chip_with_id("Zetta", "ZD25Q32C", 0xBA, 0x6016, 64, 64),
        nor_chip_with_id("Zetta", "ZD25Q64C", 0xBA, 0x6017, 128, 64),
        nor_chip_with_id("Zetta", "ZD25Q128C", 0xBA, 0x6018, 256, 64),
    ]
}

// =========================================================================
// PUYA (0x85)
// =========================================================================
fn puya_chips() -> Vec<ChipSpec> {
    vec![
        nor_chip_with_id("PUYA", "P25Q16H", 0x85, 0x6015, 32, 64),
        nor_chip_with_id("PUYA", "P25Q32H", 0x85, 0x6016, 64, 64),
        nor_chip_with_id("PUYA", "PY25Q32HB", 0x85, 0x2016, 64, 64),
        nor_chip_with_id("PUYA", "P25Q64H", 0x85, 0x6017, 128, 64),
        nor_chip_with_id("PUYA", "P25Q128H", 0x85, 0x6018, 256, 64),
        nor_chip_with_id("PUYA", "PY25Q128HA", 0x85, 0x2018, 256, 64),
    ]
}

/// Helper function to create a NOR chip spec with explicit manufacturer
fn nor_chip_with_id(
    manufacturer: &str,
    name: &str,
    mfr_id: u8,
    jedec_id: u16,
    n_sectors: u32,
    sector_size_kb: u32,
) -> ChipSpec {
    let capacity_bytes = n_sectors * sector_size_kb * 1024;

    ChipSpec {
        name: name.to_string(),
        manufacturer: manufacturer.to_string(),
        jedec_id: JedecId::new([mfr_id, (jedec_id >> 8) as u8, jedec_id as u8]),
        flash_type: FlashType::Nor,
        capacity: Capacity::bytes(capacity_bytes),
        layout: ChipLayout {
            page_size: 256,
            block_size: sector_size_kb * 1024,
            oob_size: None,
        },
        capabilities: ChipCapabilities::default(),
    }
}
