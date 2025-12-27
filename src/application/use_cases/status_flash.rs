//! Use Case - Flash Status and Protection
//!
//! Orchestrates reading and writing flash status/feature registers.

use crate::domain::FlashOperation;
use crate::error::Result;

pub struct StatusUseCase<T: FlashOperation> {
    flash: T,
}

impl<T: FlashOperation> StatusUseCase<T> {
    pub fn new(flash: T) -> Self {
        Self { flash }
    }

    /// Read raw status register(s)
    pub fn get_status(&mut self) -> Result<Vec<u8>> {
        self.flash.get_status()
    }

    /// Write raw status register(s)
    pub fn set_status(&mut self, status: &[u8]) -> Result<()> {
        self.flash.set_status(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{EraseRequest, Progress, ReadRequest, WriteRequest};
    use std::cell::RefCell;

    struct MockFlashSpy {
        pub last_status_write: RefCell<Option<Vec<u8>>>,
        pub status_to_return: Vec<u8>,
    }

    impl MockFlashSpy {
        fn new(status_to_return: Vec<u8>) -> Self {
            Self {
                last_status_write: RefCell::new(None),
                status_to_return,
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
        fn erase(&mut self, _request: EraseRequest, _on_progress: &dyn Fn(Progress)) -> Result<()> {
            Ok(())
        }

        fn get_status(&mut self) -> Result<Vec<u8>> {
            Ok(self.status_to_return.clone())
        }

        fn set_status(&mut self, status: &[u8]) -> Result<()> {
            *self.last_status_write.borrow_mut() = Some(status.to_vec());
            Ok(())
        }
    }

    #[test]
    fn test_status_use_case() {
        let flash = MockFlashSpy::new(vec![0xA5]);
        let mut use_case = StatusUseCase::new(flash);

        // Test Read
        let status = use_case.get_status().unwrap();
        assert_eq!(status, vec![0xA5]);

        // Test Write
        use_case.set_status(&[0x5A]).unwrap();
        let written = use_case.flash.last_status_write.borrow();
        assert_eq!(written.as_ref().unwrap(), &vec![0x5A]);
    }
}
