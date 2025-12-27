//! Write Flash Use Case
//!
//! Orchestrates writing data to flash memory.

use crate::domain::{
    bad_block::BadBlockTable, Address, BadBlockStrategy, FlashOperation, OobMode, Progress,
    WriteRequest,
};
use crate::error::Result;

/// Parameters for write operation
pub struct WriteParams<'a> {
    pub address: u32,
    pub data: &'a [u8],
    pub use_ecc: bool,
    pub verify: bool,
    /// Ignore ECC errors during verify (for data recovery)
    pub ignore_ecc_errors: bool,
    pub oob_mode: OobMode,
    pub bad_block_strategy: BadBlockStrategy,
    pub bbt: Option<BadBlockTable>,
    pub retry_count: u32,
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
            bbt: params.bbt,
            retry_count: params.retry_count,
        };

        self.flash.write(request, &on_progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{EraseRequest, FlashOperation, Progress, ReadRequest, WriteRequest};
    use crate::error::Result;
    use std::cell::RefCell;

    struct MockFlashSpy {
        pub last_write_request: RefCell<Option<Vec<u8>>>, // storing just data or partial req for check
        pub last_addr: RefCell<u32>,
    }

    impl MockFlashSpy {
        fn new() -> Self {
            Self {
                last_write_request: RefCell::new(None),
                last_addr: RefCell::new(0),
            }
        }
    }

    impl FlashOperation for MockFlashSpy {
        fn read(
            &mut self,
            _request: ReadRequest,
            _on_progress: &dyn Fn(Progress),
        ) -> Result<Vec<u8>> {
            Ok(vec![])
        }

        fn write(&mut self, request: WriteRequest, _on_progress: &dyn Fn(Progress)) -> Result<()> {
            *self.last_write_request.borrow_mut() = Some(request.data.to_vec());
            *self.last_addr.borrow_mut() = request.address.as_u32();
            Ok(())
        }
        fn erase(&mut self, _request: EraseRequest, _on_progress: &dyn Fn(Progress)) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_write_flash_use_case() {
        let flash = MockFlashSpy::new();
        let mut use_case = WriteFlashUseCase::new(flash);

        let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let params = WriteParams {
            address: 0x2000,
            data: &data,
            use_ecc: true,
            verify: true,
            ignore_ecc_errors: true,
            oob_mode: OobMode::None,
            bad_block_strategy: BadBlockStrategy::Fail,
            bbt: None,
            retry_count: 0,
        };

        let result = use_case.execute(params, |_| {});
        assert!(result.is_ok());

        assert_eq!(use_case.flash.last_addr.borrow().clone(), 0x2000);
        let written_data = use_case.flash.last_write_request.borrow();
        assert_eq!(written_data.as_ref().unwrap(), &data);
    }
}
