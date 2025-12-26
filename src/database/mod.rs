//! Flash chip database
//!
//! This module contains information about supported Flash chips,
//! including manufacturers, timing parameters, and layout information.

mod chips;

pub use chips::SUPPORTED_CHIPS;

/// Flash chip type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlashType {
    /// SPI NOR Flash
    Nor,
    /// SPI NAND Flash
    Nand,
}

/// Flash chip information
#[derive(Debug, Clone)]
pub struct ChipInfo {
    /// Chip name/model
    pub name: &'static str,
    /// Manufacturer name
    pub manufacturer: &'static str,
    /// JEDEC ID (3 bytes: manufacturer + device ID)
    pub jedec_id: [u8; 3],
    /// Flash type (NAND or NOR)
    pub flash_type: FlashType,
    /// Total capacity in bytes
    pub capacity: u32,
    /// Page size in bytes
    pub page_size: u32,
    /// OOB (spare) size in bytes (NAND only)
    pub oob_size: Option<u32>,
    /// Block size in bytes (for erase operations)
    pub block_size: Option<u32>,
    /// Number of blocks
    pub block_count: Option<u32>,
    /// Pages per block
    pub pages_per_block: Option<u32>,
}

impl ChipInfo {
    /// Create a new NOR chip info
    pub const fn nor(
        name: &'static str,
        manufacturer: &'static str,
        jedec_id: [u8; 3],
        capacity: u32,
        page_size: u32,
        block_size: u32,
    ) -> Self {
        Self {
            name,
            manufacturer,
            jedec_id,
            flash_type: FlashType::Nor,
            capacity,
            page_size,
            oob_size: None,
            block_size: Some(block_size),
            block_count: Some(capacity / block_size),
            pages_per_block: Some(block_size / page_size),
        }
    }

    /// Create a new NAND chip info
    pub const fn nand(
        name: &'static str,
        manufacturer: &'static str,
        jedec_id: [u8; 3],
        capacity: u32,
        page_size: u32,
        oob_size: u32,
        block_size: u32,
    ) -> Self {
        Self {
            name,
            manufacturer,
            jedec_id,
            flash_type: FlashType::Nand,
            capacity,
            page_size,
            oob_size: Some(oob_size),
            block_size: Some(block_size),
            block_count: Some(capacity / block_size),
            pages_per_block: Some(block_size / page_size),
        }
    }
}

/// Look up a chip by its JEDEC ID
pub fn lookup_chip(jedec_id: &[u8; 3]) -> Option<ChipInfo> {
    SUPPORTED_CHIPS
        .iter()
        .find(|chip| chip.jedec_id == *jedec_id)
        .cloned()
}

/// Get all supported chips
pub fn list_supported_chips() -> &'static [ChipInfo] {
    &SUPPORTED_CHIPS
}

/// Manufacturer IDs (first byte of JEDEC ID)
pub mod manufacturers {
    pub const GIGADEVICE: u8 = 0xC8;
    pub const WINBOND: u8 = 0xEF;
    pub const MACRONIX: u8 = 0xC2;
    pub const MICRON: u8 = 0x2C;
    pub const SPANSION: u8 = 0x01;
    pub const SST: u8 = 0xBF;
    pub const ISSI: u8 = 0x9D;
    pub const ESMT: u8 = 0x8C;
    pub const TOSHIBA: u8 = 0x98;
    pub const SAMSUNG: u8 = 0xEC;
    pub const HYNIX: u8 = 0xAD;
    pub const XTX: u8 = 0x0B;
    pub const DOSILICON: u8 = 0xE5;
    pub const FORESEE: u8 = 0xCD;
}
