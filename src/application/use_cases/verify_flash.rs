//! Verify Flash Use Case
//!
//! Orchestrates the verification of data written to flash memory.

use crate::domain::{
    bad_block::BadBlockTable, Address, BadBlockStrategy, FlashOperation, OobMode, Progress,
    ReadRequest,
};
use crate::error::{Error, Result};

/// Parameters for verify operation
pub struct VerifyParams<'a> {
    pub address: u32,
    pub data: &'a [u8],
    pub use_ecc: bool,
    /// Ignore ECC errors and continue verifying (for data recovery)
    pub ignore_ecc_errors: bool,
    pub oob_mode: OobMode,
    pub bad_block_strategy: BadBlockStrategy,
    pub bbt: Option<BadBlockTable>,
    pub retry_count: u32,
}

/// Use case for verifying flash contents
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
            ignore_ecc_errors: params.ignore_ecc_errors,
            oob_mode: params.oob_mode,
            bad_block_strategy: params.bad_block_strategy,
            bbt: params.bbt,
            retry_count: params.retry_count,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{EraseRequest, FlashOperation, WriteRequest};

    struct MockFlashReader {
        mock_data: Vec<u8>,
    }

    impl MockFlashReader {
        fn new(data: Vec<u8>) -> Self {
            Self { mock_data: data }
        }
    }

    impl FlashOperation for MockFlashReader {
        fn read(
            &mut self,
            request: ReadRequest,
            _on_progress: &dyn Fn(Progress),
        ) -> Result<Vec<u8>> {
            // Return slice of mock data if within bounds, else empty or error
            // For simplicity just return the whole mock data or a subset based on request
            // Here assuming request asks for correct length
            Ok(self.mock_data.clone())
        }

        fn write(&mut self, _request: WriteRequest, _on_progress: &dyn Fn(Progress)) -> Result<()> {
            Ok(())
        }
        fn erase(&mut self, _request: EraseRequest, _on_progress: &dyn Fn(Progress)) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_verify_success() {
        let data = vec![1, 2, 3, 4];
        let flash = MockFlashReader::new(data.clone());
        let mut use_case = VerifyFlashUseCase::new(flash);

        let params = VerifyParams {
            address: 0,
            data: &data,
            use_ecc: true,
            ignore_ecc_errors: false,
            oob_mode: OobMode::None,
            bad_block_strategy: BadBlockStrategy::Fail,
            bbt: None,
            retry_count: 0,
        };

        let result = use_case.execute(params, |_| {});
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_failure() {
        let actual_data = vec![1, 2, 0, 4]; // 3rd byte differs
        let expected_data = vec![1, 2, 3, 4];

        let flash = MockFlashReader::new(actual_data);
        let mut use_case = VerifyFlashUseCase::new(flash);

        let params = VerifyParams {
            address: 0x100,
            data: &expected_data,
            use_ecc: true,
            ignore_ecc_errors: false,
            oob_mode: OobMode::None,
            bad_block_strategy: BadBlockStrategy::Fail,
            bbt: None,
            retry_count: 0,
        };

        let result = use_case.execute(params, |_| {});
        assert!(result.is_err());

        match result {
            Err(Error::VerificationFailed {
                address,
                expected,
                actual,
            }) => {
                assert_eq!(address, 0x100 + 2); // 0x100 offset + index 2
                assert_eq!(expected, 3);
                assert_eq!(actual, 0);
            }
            _ => panic!("Expected VerificationFailed error"),
        }
    }
}
