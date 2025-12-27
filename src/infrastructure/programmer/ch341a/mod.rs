//! CH341A Programmer Implementation
//!
//! Infrastructure layer implementation of the CH341A USB-to-SPI bridge.

pub mod protocol;

use futures_lite::future::block_on;
use log::{debug, trace};
use nusb::transfer::RequestBuffer;

use super::traits::Programmer;
use crate::error::Result;
use protocol::SpiSpeed;

// CH341A USB identifiers
pub const CH341A_VID: u16 = 0x1A86;
pub const CH341A_PID: u16 = 0x5512;

// CH341A endpoints
const EP_OUT: u8 = 0x02;
const EP_IN: u8 = 0x82;

/// CH341A programmer instance
pub struct Ch341a {
    _device: nusb::Device, // Kept to maintain connection
    interface: nusb::Interface,
    speed: SpiSpeed,
}

impl Ch341a {
    /// Create a new CH341A instance from an opened device
    pub fn new(device: nusb::Device) -> Result<Self> {
        let interface = device.claim_interface(0)?;
        let mut ch341a = Ch341a {
            _device: device,
            interface,
            speed: SpiSpeed::Medium,
        };

        ch341a.configure_spi()?;
        Ok(ch341a)
    }

    /// Configure SPI mode
    fn configure_spi(&mut self) -> Result<()> {
        debug!("Configuring CH341A for SPI mode...");
        let cmd = protocol::build_set_mode_cmd(self.speed);
        self.bulk_write(&cmd)?;
        Ok(())
    }

    /// Perform a bulk write to the device
    fn bulk_write(&self, data: &[u8]) -> Result<()> {
        trace!("USB OUT: {:02X?}", data);
        let result = block_on(async { self.interface.bulk_out(EP_OUT, data.to_vec()).await });
        result.status?;
        Ok(())
    }

    /// Perform a bulk read from the device
    fn bulk_read(&self, len: usize) -> Result<Vec<u8>> {
        let result =
            block_on(async { self.interface.bulk_in(EP_IN, RequestBuffer::new(len)).await });
        let data = result.into_result()?;
        trace!("USB IN: {:02X?}", data);
        Ok(data)
    }

    /// Set SPI speed internally
    fn set_speed_internal(&mut self, speed: SpiSpeed) -> Result<()> {
        self.speed = speed;
        self.configure_spi()
    }
}

impl Programmer for Ch341a {
    fn name(&self) -> &str {
        "CH341A USB Programmer"
    }

    fn spi_transfer(&mut self, tx_data: &[u8], rx_data: &mut [u8]) -> Result<()> {
        if tx_data.is_empty() {
            return Ok(());
        }

        // CH341A has a 32-byte SPI transfer limit per packet for standard transfer
        for (tx_chunk, rx_chunk) in tx_data.chunks(32).zip(rx_data.chunks_mut(32)) {
            let cmd = protocol::build_spi_transfer_cmd(tx_chunk);
            self.bulk_write(&cmd)?;
            let response = self.bulk_read(tx_chunk.len())?;
            rx_chunk.copy_from_slice(&response);
        }

        Ok(())
    }

    fn set_cs(&mut self, active: bool) -> Result<()> {
        let cmd = protocol::build_cs_cmd(active);
        self.bulk_write(&cmd)?;
        Ok(())
    }

    /// Optimized bulk SPI read for large data transfers.
    ///
    /// Uses larger USB packets to reduce USB overhead significantly.
    /// For a 2KB page read:
    /// - Standard mode: 64 USB transactions
    /// - Bulk mode: ~1 USB transaction
    fn spi_read_bulk(&mut self, len: usize) -> Result<Vec<u8>> {
        if len == 0 {
            return Ok(Vec::new());
        }

        let mut result = Vec::with_capacity(len);

        // Use larger chunks for bulk transfer (up to 4KB per USB transaction)
        for chunk_size in
            std::iter::repeat(protocol::MAX_SPI_STREAM_SIZE).scan(len, |remaining, chunk| {
                if *remaining == 0 {
                    None
                } else {
                    let size = (*remaining).min(chunk);
                    *remaining -= size;
                    Some(size)
                }
            })
        {
            let cmd = protocol::build_spi_stream_cmd(chunk_size);
            self.bulk_write(&cmd)?;
            let response = self.bulk_read(chunk_size)?;
            result.extend_from_slice(&response);
        }

        Ok(result)
    }

    /// Execute a complete SPI transaction with embedded CS control.
    ///
    /// More efficient than separate calls as it reduces USB round-trips.
    fn spi_transaction(&mut self, tx: &[u8], rx_len: usize) -> Result<Vec<u8>> {
        self.set_cs(true)?;

        // Write command/address
        if !tx.is_empty() {
            self.spi_write(tx)?;
        }

        // Read response using bulk method for large reads
        let rx = if rx_len > protocol::MAX_SPI_TRANSFER_SIZE * 2 {
            self.spi_read_bulk(rx_len)?
        } else {
            self.spi_read(rx_len)?
        };

        self.set_cs(false)?;
        Ok(rx)
    }

    fn spi_transaction_write(&mut self, tx: &[u8]) -> Result<()> {
        self.set_cs(true)?;
        self.spi_write(tx)?;
        self.set_cs(false)?;
        Ok(())
    }

    fn max_bulk_transfer_size(&self) -> usize {
        protocol::MAX_SPI_STREAM_SIZE
    }

    fn set_speed(&mut self, speed: u8) -> Result<()> {
        let spi_speed = SpiSpeed::from_u8(speed);
        debug!("Setting SPI speed to: {}", spi_speed.description());
        self.set_speed_internal(spi_speed)
    }
}
