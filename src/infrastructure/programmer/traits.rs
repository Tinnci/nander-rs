//! Infrastructure - Programmer Trait
//!
//! Abstract interface for hardware programmers.

use crate::error::Result;

/// Default bulk transfer chunk size (32KB for optimal USB throughput)
pub const DEFAULT_BULK_CHUNK_SIZE: usize = 32 * 1024;

/// Traits defining the interface for hardware programmers
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

    // =========================================================================
    // Optimized Bulk Transfer Methods (with default implementations)
    // =========================================================================

    /// Bulk SPI read optimized for large data transfers.
    ///
    /// Default implementation falls back to regular `spi_read`.
    /// Hardware drivers should override this for optimal performance.
    fn spi_read_bulk(&mut self, len: usize) -> Result<Vec<u8>> {
        self.spi_read(len)
    }

    /// Execute a complete SPI transaction with embedded CS control.
    ///
    /// This is more efficient than separate CS + write + read + CS calls
    /// as it can be optimized into a single USB transfer.
    ///
    /// # Arguments
    /// * `tx` - Data to transmit
    /// * `rx_len` - Number of bytes to read after transmitting
    ///
    /// # Returns
    /// The received data bytes
    fn spi_transaction(&mut self, tx: &[u8], rx_len: usize) -> Result<Vec<u8>> {
        // Default implementation: use separate calls
        self.set_cs(true)?;
        self.spi_write(tx)?;
        let rx = self.spi_read(rx_len)?;
        self.set_cs(false)?;
        Ok(rx)
    }

    /// Execute a write-only SPI transaction with embedded CS control.
    fn spi_transaction_write(&mut self, tx: &[u8]) -> Result<()> {
        self.set_cs(true)?;
        self.spi_write(tx)?;
        self.set_cs(false)?;
        Ok(())
    }

    /// Get the maximum supported bulk transfer size for this programmer.
    ///
    /// This helps Flash protocol implementations optimize their read/write strategies.
    fn max_bulk_transfer_size(&self) -> usize {
        DEFAULT_BULK_CHUNK_SIZE
    }

    /// Set SPI clock speed (if supported by the programmer).
    ///
    /// The interpretation of the `speed` parameter may vary by programmer.
    /// For CH341A, it's a value from 0 to 7.
    fn set_speed(&mut self, _speed: u8) -> Result<()> {
        Ok(())
    }
}

impl Programmer for Box<dyn Programmer> {
    fn name(&self) -> &str {
        self.as_ref().name()
    }

    fn spi_transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<()> {
        self.as_mut().spi_transfer(tx, rx)
    }

    fn spi_write(&mut self, data: &[u8]) -> Result<()> {
        self.as_mut().spi_write(data)
    }

    fn spi_read(&mut self, len: usize) -> Result<Vec<u8>> {
        self.as_mut().spi_read(len)
    }

    fn set_cs(&mut self, active: bool) -> Result<()> {
        self.as_mut().set_cs(active)
    }

    fn spi_read_bulk(&mut self, len: usize) -> Result<Vec<u8>> {
        self.as_mut().spi_read_bulk(len)
    }

    fn spi_transaction(&mut self, tx: &[u8], rx_len: usize) -> Result<Vec<u8>> {
        self.as_mut().spi_transaction(tx, rx_len)
    }

    fn spi_transaction_write(&mut self, tx: &[u8]) -> Result<()> {
        self.as_mut().spi_transaction_write(tx)
    }

    fn max_bulk_transfer_size(&self) -> usize {
        self.as_ref().max_bulk_transfer_size()
    }

    fn set_speed(&mut self, speed: u8) -> Result<()> {
        self.as_mut().set_speed(speed)
    }
}
