//! Erase Flash Use Case
//!
//! Orchestrates erasing flash memory blocks.

use crate::domain::{
    bad_block::BadBlockTable, Address, BadBlockStrategy, EraseRequest, FlashOperation, Progress,
};
use crate::error::Result;

/// Parameters for erase operation
pub struct EraseParams {
    pub address: u32,
    pub length: u32,
    pub bad_block_strategy: BadBlockStrategy,
    pub bbt: Option<BadBlockTable>,
}

/// Use case for erasing data from flash
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
            bbt: params.bbt,
        };

        self.flash.erase(request, &on_progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{FlashOperation, Progress, ReadRequest, WriteRequest};
    use crate::error::Result;
    use std::cell::RefCell;

    struct MockFlashSpy {
        pub last_erase_request: RefCell<Option<EraseRequest>>,
    }

    impl MockFlashSpy {
        fn new() -> Self {
            Self {
                last_erase_request: RefCell::new(None),
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

        fn write(&mut self, _request: WriteRequest, _on_progress: &dyn Fn(Progress)) -> Result<()> {
            Ok(())
        }

        fn erase(&mut self, request: EraseRequest, _on_progress: &dyn Fn(Progress)) -> Result<()> {
            *self.last_erase_request.borrow_mut() = Some(request);
            Ok(())
        }
    }

    #[test]
    fn test_erase_flash_use_case() {
        let flash = MockFlashSpy::new();
        let mut use_case = EraseFlashUseCase::new(flash);

        let params = EraseParams {
            address: 0x4000,
            length: 128 * 1024,
            bad_block_strategy: BadBlockStrategy::Skip,
            bbt: None,
        };

        let result = use_case.execute(params, |_| {});
        assert!(result.is_ok());

        assert!(use_case.flash.last_erase_request.borrow().is_some());
        let req = use_case.flash.last_erase_request.borrow();
        let req = req.as_ref().unwrap();

        assert_eq!(req.address.as_u32(), 0x4000);
        assert_eq!(req.length, 128 * 1024);
        assert_eq!(req.bad_block_strategy, BadBlockStrategy::Skip);
    }
}
