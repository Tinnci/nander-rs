//! Read Flash Use Case
//!
//! Orchestrates reading data from flash memory.

use crate::domain::{
    bad_block::BadBlockTable, Address, BadBlockStrategy, FlashOperation, OobMode, Progress,
    ReadRequest,
};
use crate::error::Result;

/// Parameters for read operation
pub struct ReadParams {
    pub address: u32,
    pub length: u32,
    pub use_ecc: bool,
    /// Ignore ECC errors and continue reading (for data recovery)
    pub ignore_ecc_errors: bool,
    pub oob_mode: OobMode,
    pub bad_block_strategy: BadBlockStrategy,
    pub bbt: Option<BadBlockTable>,
}

/// Use case for reading data from flash
pub struct ReadFlashUseCase<F: FlashOperation> {
    flash: F,
}

impl<F: FlashOperation> ReadFlashUseCase<F> {
    /// Create a new read flash use case
    pub fn new(flash: F) -> Self {
        Self { flash }
    }

    /// Execute the read operation
    pub fn execute<P>(&mut self, params: ReadParams, on_progress: P) -> Result<Vec<u8>>
    where
        P: Fn(Progress),
    {
        let request = ReadRequest {
            address: Address::new(params.address),
            length: params.length,
            use_ecc: params.use_ecc,
            ignore_ecc_errors: params.ignore_ecc_errors,
            oob_mode: params.oob_mode,
            bad_block_strategy: params.bad_block_strategy,
            bbt: params.bbt,
            retry_count: 0,
        };

        self.flash.read(request, &on_progress)
    }
}
