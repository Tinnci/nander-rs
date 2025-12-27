//! CH347 Programmer Implementation
//!
//! Official VID: 1A86, PID: 55DB (Mode 1: UART + SPI + I2C)

use crate::error::{Error, Result};
use crate::infrastructure::programmer::traits::Programmer;
use log::{debug, warn};
use nusb::Device;

pub const CH347_VID: u16 = 0x1A86;
pub const CH347_PID: u16 = 0x55DB;

#[allow(dead_code)]
mod commands {
    pub const CMD_SPI_SET_CFG: u8 = 0xC0; // Configure SPI
    pub const CMD_SPI_CONTROL: u8 = 0xC1; // CS control
    pub const CMD_SPI_RD_WR: u8 = 0xC2; // Standard read/write
    pub const CMD_SPI_BLCK_RD: u8 = 0xC3; // Block read
    pub const CMD_SPI_BLCK_WR: u8 = 0xC4; // Block write
}

pub struct Ch347 {
    #[allow(dead_code)]
    device: Device,
    interface: nusb::Interface,
    out_endpoint: u8,
    in_endpoint: u8,
    current_speed: u8,
}

impl Ch347 {
    pub fn new(device: Device) -> Result<Self> {
        // CH347 in Mode 1 (UART+SPI+I2C) uses several interfaces.
        // SPI/I2C/GPIO is typically on Interface 1 (Vendor Specific).
        let interface_index = 1;

        let interface = device.claim_interface(interface_index).map_err(|e| {
            Error::Other(format!(
                "Failed to claim CH347 interface {}: {}",
                interface_index, e
            ))
        })?;

        // Standard CH347 endpoints for SPI mode
        let out_endpoint = 0x01;
        let in_endpoint = 0x81;

        let mut programmer = Self {
            device,
            interface,
            out_endpoint,
            in_endpoint,
            current_speed: 5, // Default ~3MHz
        };

        // Initialize with default settings
        programmer.set_speed(5)?;

        Ok(programmer)
    }

    fn usb_write_read(&mut self, write_buf: &[u8], read_len: usize) -> Result<Vec<u8>> {
        use futures_lite::future::block_on;
        use nusb::transfer::RequestBuffer;

        // 1. Write command
        let write_result = block_on(
            self.interface
                .bulk_out(self.out_endpoint, write_buf.to_vec()),
        );
        write_result.status?;

        if read_len == 0 {
            return Ok(Vec::new());
        }

        // 2. Read response
        let read_result = block_on(
            self.interface
                .bulk_in(self.in_endpoint, RequestBuffer::new(read_len)),
        );
        let data = read_result.into_result()?;

        Ok(data)
    }
}

impl Programmer for Ch347 {
    fn name(&self) -> &str {
        "CH347 High-Speed Programmer"
    }

    fn spi_transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<()> {
        if tx.is_empty() {
            return Ok(());
        }

        // CH347 RD_WR format: [CMD_SPI_RD_WR(1B)] + [Length(2B)] + [Payload]
        let mut packet = Vec::with_capacity(3 + tx.len());
        packet.push(commands::CMD_SPI_RD_WR);
        packet.push((tx.len() & 0xFF) as u8);
        packet.push(((tx.len() >> 8) & 0xFF) as u8);
        packet.extend_from_slice(tx);

        let response = self.usb_write_read(&packet, tx.len())?;

        if response.len() != rx.len() {
            warn!(
                "CH347: SPI transfer length mismatch (Expected {}, got {})",
                rx.len(),
                response.len()
            );
        }

        let copy_len = response.len().min(rx.len());
        rx[..copy_len].copy_from_slice(&response[..copy_len]);

        Ok(())
    }

    fn set_cs(&mut self, active: bool) -> Result<()> {
        // CMD_SPI_CONTROL (0xC1) payload: [CS_Level]
        // Usually 0 for active (low), 1 for inactive (high).
        let cs_level = if active { 0 } else { 1 };
        let packet = [commands::CMD_SPI_CONTROL, 1, 0, cs_level];
        self.usb_write_read(&packet, 0)?;
        Ok(())
    }

    fn set_speed(&mut self, speed: u8) -> Result<()> {
        debug!("CH347: Setting speed level {}", speed);

        // CH347 Config Packet (simplified based on research)
        // CMD_SPI_SET_CFG (0xC0) followed by 26 bytes of configuration data.
        let mut cfg = vec![0u8; 26];

        // Value mapping for iClock (Index 5):
        // 0: 60MHz, 1: 30MHz, 2: 15MHz, 3: 7.5MHz, 4: 3.75MHz, 5: 1.875MHz, 6: 937.5KHz, 7: 468.75KHz
        cfg[5] = speed.min(7);

        // Mode (Index 1): SPI Mode 0 as default
        cfg[1] = 0;

        // Byte Order (Index 7): 1 = MSB First
        cfg[7] = 1;

        let mut packet = Vec::with_capacity(3 + cfg.len());
        packet.push(commands::CMD_SPI_SET_CFG);
        packet.push((cfg.len() & 0xFF) as u8);
        packet.push(((cfg.len() >> 8) & 0xFF) as u8);
        packet.extend_from_slice(&cfg);

        self.usb_write_read(&packet, 0)?;
        self.current_speed = speed;
        Ok(())
    }
}
