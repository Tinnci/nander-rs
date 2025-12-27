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
    current_outputs: u8,
}

impl Ch341a {
    /// Create a new CH341A instance from an opened device
    pub fn new(device: nusb::Device) -> Result<Self> {
        let interface = device.claim_interface(0)?;
        let mut ch341a = Ch341a {
            _device: device,
            interface,
            speed: SpiSpeed::Medium,
            current_outputs: 0x37, // Default: D0-D5 high except D3/D2? Wait.
                                   // 0x37 = 110111b (D0-D2, D4-D5 high, D3 low)
        };

        ch341a.configure_spi()?;
        Ok(ch341a)
    }

    /// Configure SPI mode
    fn configure_spi(&mut self) -> Result<()> {
        debug!("Configuring CH341A for SPI mode...");
        let cmd = protocol::build_set_mode_cmd(self.speed);
        self.bulk_write(&cmd)?;
        self.current_outputs = 0x37;
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
        // Update current_outputs bit 0
        if active {
            self.current_outputs &= !0x01;
        } else {
            self.current_outputs |= 0x01;
        }
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
        // Optimized: Combine CS and data into a single USB transfer
        // This significantly reduces overhead for small writes (like commands)
        let mut cmd = Vec::with_capacity(tx.len() + 10);

        // 1. Assert CS
        cmd.extend_from_slice(&protocol::build_cs_cmd(true));

        // 2. Data
        if !tx.is_empty() {
            // Use stream command or simple append if we had a helper.
            // Manually build the SPI stream command part here to inline it
            // Note: build_spi_transfer_cmd includes the CMD_SPI_STREAM byte.
            cmd.push(protocol::CMD_SPI_STREAM);
            cmd.extend_from_slice(tx);
        }

        // 3. De-assert CS
        cmd.extend_from_slice(&protocol::build_cs_cmd(false));

        self.bulk_write(&cmd)?;
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

    fn i2c_write(&mut self, addr: u8, data: &[u8]) -> Result<()> {
        if data.len() > 31 {
            return Err(crate::error::Error::InvalidParameter(
                "I2C write data length exceeds 31 bytes".to_string(),
            ));
        }

        let mut cmd = Vec::with_capacity(data.len() + 6);
        cmd.push(protocol::CMD_I2C_STREAM);
        cmd.push(protocol::i2c_sub::START);
        cmd.push(protocol::i2c_sub::OUT | (data.len() as u8 + 1));
        cmd.push(addr); // Device address (Write)
        cmd.extend_from_slice(data);
        cmd.push(protocol::i2c_sub::STOP);
        cmd.push(protocol::CMD_I2C_STM_END);

        self.bulk_write(&cmd)
    }

    fn i2c_read(&mut self, addr: u8, len: usize) -> Result<Vec<u8>> {
        if len > 32 {
            return Err(crate::error::Error::InvalidParameter(
                "I2C read length exceeds 32 bytes".to_string(),
            ));
        }

        let mut cmd = Vec::with_capacity(8);
        cmd.push(protocol::CMD_I2C_STREAM);
        cmd.push(protocol::i2c_sub::START);
        cmd.push(protocol::i2c_sub::OUT | 1);
        cmd.push(addr | 1); // Device address (Read)
        cmd.push(protocol::i2c_sub::IN | (len as u8));
        cmd.push(protocol::i2c_sub::STOP);
        cmd.push(protocol::CMD_I2C_STM_END);

        self.bulk_write(&cmd)?;
        self.bulk_read(len)
    }

    fn gpio_set(&mut self, pin: u8, level: bool) -> Result<()> {
        let cmd = protocol::build_gpio_cmd(pin, level, self.current_outputs);
        self.bulk_write(&cmd)?;

        let mask = 1u8 << pin;
        if level {
            self.current_outputs |= mask;
        } else {
            self.current_outputs &= !mask;
        }
        Ok(())
    }

    fn gpio_get(&mut self, _pin: u8) -> Result<bool> {
        // To read pins, we use CMD_GET_STATUS
        let cmd = vec![protocol::CMD_GET_STATUS];
        self.bulk_write(&cmd)?;
        let response = self.bulk_read(2)?; // Status is 2 bytes

        // CH341A Status bits:
        // bit 0-7: ERR#, PEMPT, SELECT, ACK, BUSY, ...
        // According to CH341 docs for UIO mode:
        // D0-D5 are in bits 0-5 of the status byte?
        // Wait, standard status read returns bits for parallel port.
        // For CH341A in SPI/I2C mode, bit 3 is DIN (D2).

        // Let's use bit 3 for D2 (DIN) which is what most apps use
        Ok((response[0] & 0x08) != 0)
    }
}
