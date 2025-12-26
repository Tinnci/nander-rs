//! Hardware abstraction layer
//!
//! This module defines the `Programmer` trait and implements
//! drivers for various hardware programmers.

pub mod ch341a;

use crate::error::Result;

/// Core trait for hardware programmers
///
/// This trait abstracts the low-level USB communication,
/// allowing support for different programmer hardware.
pub trait Programmer: Sized {
    /// Open a connection to the programmer
    fn open() -> Result<Self>;

    /// Close the connection
    fn close(&mut self) -> Result<()>;

    /// Perform a SPI transfer (simultaneous write and read)
    ///
    /// # Arguments
    /// * `tx_data` - Data to transmit
    /// * `rx_data` - Buffer to receive data into
    fn spi_transfer(&mut self, tx_data: &[u8], rx_data: &mut [u8]) -> Result<()>;

    /// Write data via SPI (ignore received data)
    fn spi_write(&mut self, data: &[u8]) -> Result<()> {
        let mut dummy = vec![0u8; data.len()];
        self.spi_transfer(data, &mut dummy)
    }

    /// Read data via SPI (send dummy bytes)
    fn spi_read(&mut self, len: usize) -> Result<Vec<u8>> {
        let tx = vec![0x00u8; len];
        let mut rx = vec![0u8; len];
        self.spi_transfer(&tx, &mut rx)?;
        Ok(rx)
    }

    /// Set chip select state
    fn set_cs(&mut self, active: bool) -> Result<()>;

    /// Read JEDEC ID from the connected flash chip
    fn read_jedec_id(&mut self) -> Result<[u8; 3]> {
        // Standard JEDEC ID command: 0x9F
        self.set_cs(true)?;
        self.spi_write(&[0x9F])?;
        let id_bytes = self.spi_read(3)?;
        self.set_cs(false)?;

        Ok([id_bytes[0], id_bytes[1], id_bytes[2]])
    }

    /// Set GPIO pin state (for Hold, WP, etc.)
    fn set_gpio(&mut self, pin: u8, level: bool) -> Result<()>;
}

/// Programmer speed settings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiSpeed {
    /// ~1.5 MHz
    Low,
    /// ~3 MHz (default)
    Medium,
    /// ~6 MHz
    High,
    /// ~12 MHz (may not work with all chips)
    VeryHigh,
}

impl Default for SpiSpeed {
    fn default() -> Self {
        Self::Medium
    }
}
