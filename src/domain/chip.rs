//! Domain Model - Chip Specification
//!
//! This module defines what a Flash chip "is" and what it can do,
//! independent of how it's programmed.

use super::types::{Capacity, FlashType, JedecId};

/// Core specification of a flash chip
#[derive(Debug, Clone)]
pub struct ChipSpec {
    pub name: String,
    pub manufacturer: String,
    pub jedec_id: JedecId,
    pub flash_type: FlashType,
    pub capacity: Capacity,
    pub layout: ChipLayout,
    pub capabilities: ChipCapabilities,
}

/// Physical layout of the chip
#[derive(Debug, Clone, Copy)]
pub struct ChipLayout {
    pub page_size: u32,
    pub block_size: u32,
    pub oob_size: Option<u32>,
}

impl ChipLayout {
    pub fn pages_per_block(&self) -> u32 {
        self.block_size / self.page_size
    }

    pub fn total_pages(&self, capacity: Capacity) -> u32 {
        capacity.as_bytes() / self.page_size
    }
}

/// What the chip supports
#[derive(Debug, Clone, Copy, Default)]
pub struct ChipCapabilities {
    pub supports_ecc_control: bool,
    pub supports_4byte_addr: bool,
    pub supports_quad_spi: bool,
    pub supports_dual_spi: bool,
}

/// Bad block management status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockStatus {
    Bad,
    Reserved,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::Capacity;

    #[test]
    fn test_chip_layout_calculations() {
        let layout = ChipLayout {
            page_size: 2048,
            block_size: 128 * 1024, // 128KB
            oob_size: Some(64),
        };

        // Pages per block: 128KB / 2KB = 64
        assert_eq!(layout.pages_per_block(), 64);

        // Total pages for 128MB capacity
        // 128MB = 134,217,728 bytes
        // 134217728 / 2048 = 65536 pages
        let capacity = Capacity::megabytes(128);
        assert_eq!(layout.total_pages(capacity), 65536);
    }
}
