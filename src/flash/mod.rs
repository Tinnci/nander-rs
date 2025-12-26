//! Flash chip protocol implementations
//!
//! This module contains SPI Flash protocol handlers for both
//! NAND and NOR flash types.

pub mod commands;
pub mod nand;
pub mod nor;

use crate::error::Result;

/// Common operations for all Flash types
pub trait FlashOps {
    /// Erase a block at the given address
    fn erase_block(&mut self, address: u32) -> Result<()>;

    /// Read a page from the flash
    fn read_page(&mut self, page: u32, buffer: &mut [u8]) -> Result<()>;

    /// Write a page to the flash
    fn write_page(&mut self, page: u32, data: &[u8]) -> Result<()>;

    /// Wait for the flash to become ready
    fn wait_ready(&mut self) -> Result<()>;

    /// Read status register
    fn read_status(&mut self) -> Result<u8>;
}

/// SPI NAND specific operations
pub trait NandFlash: FlashOps {
    /// Read page data with OOB (Out-Of-Band) area
    fn read_page_with_oob(&mut self, page: u32, data: &mut [u8], oob: &mut [u8]) -> Result<()>;

    /// Write page data with OOB area
    fn write_page_with_oob(&mut self, page: u32, data: &[u8], oob: &[u8]) -> Result<()>;

    /// Check if a block is bad
    fn is_bad_block(&mut self, block: u32) -> Result<bool>;

    /// Mark a block as bad
    fn mark_bad_block(&mut self, block: u32) -> Result<()>;
}

/// SPI NOR specific operations
pub trait NorFlash: FlashOps {
    /// Erase a sector (typically 4KB)
    fn erase_sector(&mut self, address: u32) -> Result<()>;

    /// Erase entire chip
    fn chip_erase(&mut self) -> Result<()>;

    /// Read data from an arbitrary address
    fn read(&mut self, address: u32, buffer: &mut [u8]) -> Result<()>;

    /// Write data to an arbitrary address (handles page boundaries)
    fn write(&mut self, address: u32, data: &[u8]) -> Result<()>;
}
