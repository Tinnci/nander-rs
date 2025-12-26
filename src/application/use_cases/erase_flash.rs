//! Erase Flash Use Case
//!
//! Orchestrates erasing flash memory blocks.

use crate::domain::{Address, BadBlockStrategy, EraseRequest, FlashOperation, Progress};
use crate::error::Result;

/// Parameters for erase operation
pub struct EraseParams {
    pub address: u32,
    pub length: u32,
    pub bad_block_strategy: BadBlockStrategy,
}

/// Use case for erasing flash memory
pub struct EraseFlashUseCase<F: FlashOperation> {
    flash: F,
}

impl<F: FlashOperation> EraseFlashUseCase<F> {
    /// Create a new erase flash use case
    pub fn new(flash: F) -> Self {
        Self { flash }
    }

    /// Execute the erase operation
    pub fn execute<P>(&mut self, params: EraseParams, on_progress: P) -> Result<()>
    where
        P: Fn(Progress),
    {
        let request = EraseRequest {
            address: Address::new(params.address),
            length: params.length,
            bad_block_strategy: params.bad_block_strategy,
        };

        self.flash.erase(request, &on_progress)
    }
}
