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
    pub retry_count: u32,
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
            retry_count: params.retry_count,
        };

        self.flash.read(request, &on_progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{EraseRequest, FlashOperation, Progress, WriteRequest};
    use crate::error::Result;
    use std::cell::RefCell;

    struct MockFlashSpy {
        pub last_read_request: RefCell<Option<ReadRequest>>,
    }

    impl MockFlashSpy {
        fn new() -> Self {
            Self {
                last_read_request: RefCell::new(None),
            }
        }
    }

    impl FlashOperation for MockFlashSpy {
        fn read(
            &mut self,
            request: ReadRequest,
            _on_progress: &dyn Fn(Progress),
        ) -> Result<Vec<u8>> {
            *self.last_read_request.borrow_mut() = Some(request);
            Ok(vec![0xAA; 10])
        }

        fn write(&mut self, _request: WriteRequest, _on_progress: &dyn Fn(Progress)) -> Result<()> {
            Ok(())
        }
        fn erase(&mut self, _request: EraseRequest, _on_progress: &dyn Fn(Progress)) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_read_flash_use_case() {
        let flash = MockFlashSpy::new();
        let mut use_case = ReadFlashUseCase::new(flash);

        let params = ReadParams {
            address: 0x1000,
            length: 2048,
            use_ecc: true,
            ignore_ecc_errors: false,
            oob_mode: OobMode::Included,
            bad_block_strategy: BadBlockStrategy::Skip,
            bbt: None,
            retry_count: 3,
        };

        let result = use_case.execute(params, |_| {});
        assert!(result.is_ok());

        assert!(use_case.flash.last_read_request.borrow().is_some());
        let req = use_case.flash.last_read_request.borrow();
        let req = req.as_ref().unwrap();

        assert_eq!(req.address.as_u32(), 0x1000);
        assert_eq!(req.length, 2048);
        assert_eq!(req.use_ecc, true);
        assert_eq!(req.oob_mode, OobMode::Included);
        assert_eq!(req.bad_block_strategy, BadBlockStrategy::Skip);
        assert_eq!(req.retry_count, 3);
    }
}
