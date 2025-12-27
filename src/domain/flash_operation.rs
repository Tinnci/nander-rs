//! Domain Model - Flash Operations
//!
//! Definintions of core flash operations as seen by the application.

use super::bad_block::{BadBlockStrategy, BadBlockTable};
use super::types::{Address, Progress};
use crate::error::Result;

/// Request for a read operation
pub struct ReadRequest {
    pub address: Address,
    pub length: u32,
    pub use_ecc: bool,
    /// Ignore ECC errors and continue reading (for data recovery)
    pub ignore_ecc_errors: bool,
    pub oob_mode: OobMode,
    pub bad_block_strategy: BadBlockStrategy,
}

/// How to handle Out Of Band data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OobMode {
    /// Ignore OOB data, read only main page
    None,
    /// Read OOB data alongside main page
    Included,
    /// Read ONLY OOB data
    Only,
}

/// Request for a write operation
pub struct WriteRequest<'a> {
    pub address: Address,
    pub data: &'a [u8],
    pub use_ecc: bool,
    pub verify: bool,
    /// Ignore ECC errors during verification read back
    pub ignore_ecc_errors: bool,
    pub oob_mode: OobMode,
    pub bad_block_strategy: BadBlockStrategy,
}

/// Request for an erase operation
pub struct EraseRequest {
    pub address: Address,
    pub length: u32,
    pub bad_block_strategy: BadBlockStrategy,
}

/// Service Trait for Flash Operations
/// This will be implemented by the Infrastructure layer (Protocols)
pub trait FlashOperation {
    fn read(&mut self, request: ReadRequest, on_progress: &dyn Fn(Progress)) -> Result<Vec<u8>>;
    fn write(&mut self, request: WriteRequest, on_progress: &dyn Fn(Progress)) -> Result<()>;
    fn erase(&mut self, request: EraseRequest, on_progress: &dyn Fn(Progress)) -> Result<()>;

    /// Read status register(s)
    fn get_status(&mut self) -> Result<Vec<u8>> {
        Err(crate::error::Error::NotSupported(
            "get_status not implemented".to_string(),
        ))
    }

    /// Write status register(s)
    fn set_status(&mut self, _status: &[u8]) -> Result<()> {
        Err(crate::error::Error::NotSupported(
            "set_status not implemented".to_string(),
        ))
    }

    /// Scan for bad blocks and return a BadBlockTable
    fn scan_bbt(&mut self, _on_progress: &dyn Fn(Progress)) -> Result<BadBlockTable> {
        // Default implementation returns an empty table (or should error?)
        // For devices that don't support BBT (like EEPROM/NOR), effectively no bad blocks.
        // But better to return NotSupported for now.
        Err(crate::error::Error::NotSupported(
            "BBT scan not implemented".to_string(),
        ))
    }
}
