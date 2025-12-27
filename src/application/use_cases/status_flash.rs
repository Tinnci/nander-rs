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
