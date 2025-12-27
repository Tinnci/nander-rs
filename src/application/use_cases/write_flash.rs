//! Write Flash Use Case
//!
//! Orchestrates writing data to flash memory.

use crate::domain::{Address, BadBlockStrategy, FlashOperation, OobMode, Progress, WriteRequest};
use crate::error::Result;

/// Parameters for write operation
pub struct WriteParams<'a> {
    pub address: u32,
    pub data: &'a [u8],
    pub use_ecc: bool,
    pub verify: bool,
    /// Ignore ECC errors during verification read back
    pub ignore_ecc_errors: bool,
    pub oob_mode: OobMode,
    pub bad_block_strategy: BadBlockStrategy,
}

/// Use case for writing data to flash
pub struct WriteFlashUseCase<F: FlashOperation> {
    flash: F,
}

impl<F: FlashOperation> WriteFlashUseCase<F> {
    /// Create a new write flash use case
    pub fn new(flash: F) -> Self {
        Self { flash }
    }

    /// Execute the write operation
    pub fn execute<P>(&mut self, params: WriteParams, on_progress: P) -> Result<()>
    where
        P: Fn(Progress),
    {
        let request = WriteRequest {
            address: Address::new(params.address),
            data: params.data,
            use_ecc: params.use_ecc,
            verify: params.verify,
            ignore_ecc_errors: params.ignore_ecc_errors,
            oob_mode: params.oob_mode,
            bad_block_strategy: params.bad_block_strategy,
        };

        self.flash.write(request, &on_progress)
    }
}
