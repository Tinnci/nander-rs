//! Infrastructure - Programmer Trait
//!
//! Abstract interface for hardware programmers.

use crate::error::Result;

pub trait Programmer {
    /// Get the identification name of the programmer
    fn name(&self) -> &str;

    /// Execute a standard SPI transfer (Bidirectional)
    fn spi_transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<()>;

    /// Simple SPI write
    fn spi_write(&mut self, data: &[u8]) -> Result<()> {
        let mut sink = vec![0u8; data.len()];
        self.spi_transfer(data, &mut sink)
    }

    /// Simple SPI read
    fn spi_read(&mut self, len: usize) -> Result<Vec<u8>> {
        let tx = vec![0xFF; len];
        let mut rx = vec![0u8; len];
        self.spi_transfer(&tx, &mut rx)?;
        Ok(rx)
    }

    /// Control the Chip Select (CS) line
    fn set_cs(&mut self, active: bool) -> Result<()>;
}
