//! Verify Flash Use Case
//!
//! Orchestrates the verification of data written to flash memory.

use crate::domain::{Address, BadBlockStrategy, FlashOperation, OobMode, Progress, ReadRequest};
use crate::error::{Error, Result};

/// Parameters for verify operation
pub struct VerifyParams<'a> {
    pub address: u32,
    pub data: &'a [u8],
    pub use_ecc: bool,
    pub oob_mode: OobMode,
    pub bad_block_strategy: BadBlockStrategy,
}

/// Use case for verifying data in flash memory
pub struct VerifyFlashUseCase<F: FlashOperation> {
    flash: F,
}

impl<F: FlashOperation> VerifyFlashUseCase<F> {
    /// Create a new verify flash use case
    pub fn new(flash: F) -> Self {
        Self { flash }
    }

    /// Execute the verification
    pub fn execute<P>(&mut self, params: VerifyParams, on_progress: P) -> Result<()>
    where
        P: Fn(Progress),
    {
        let request = ReadRequest {
            address: Address::new(params.address),
            length: params.data.len() as u32,
            use_ecc: params.use_ecc,
            oob_mode: params.oob_mode,
            bad_block_strategy: params.bad_block_strategy,
        };

        // Read back the data from flash
        let actual_data = self.flash.read(request, &on_progress)?;

        // Compare with expected data
        if actual_data == params.data {
            Ok(())
        } else {
            // Find first discrepancy
            for (i, (a, e)) in actual_data.iter().zip(params.data.iter()).enumerate() {
                if a != e {
                    return Err(Error::VerificationFailed {
                        address: params.address + i as u32,
                        expected: *e,
                        actual: *a,
                    });
                }
            }
            // Fallback for length mismatch
            Err(Error::InvalidParameter("Data lengths differ".to_string()))
        }
    }
}
