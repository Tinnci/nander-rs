//! CH347 Programmer Implementation
//!
//! Official VID: 1A86, PID: 55DB (Mode 1: UART + SPI + I2C)

pub mod protocol;

use crate::error::{Error, Result};
use crate::infrastructure::programmer::traits::Programmer;
use log::debug;
use nusb::Device;

pub const CH347_VID: u16 = 0x1A86;
pub const CH347_PID: u16 = 0x55DB;

pub struct Ch347 {
    #[allow(dead_code)]
    device: Device,
    interface: nusb::Interface,
    out_endpoint: u8,
    in_endpoint: u8,
    #[allow(dead_code)]
    current_speed: u8,
    larger_pack_supported: bool,
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
            current_speed: 5, // Default ~1.8MHz
            larger_pack_supported: false,
        };

        // 1. Try to enable Larger Pack mode for better performance
        programmer.try_enable_larger_pack();

        // 2. Initialize with default settings
        programmer.set_speed(5)?;

        Ok(programmer)
    }

    fn try_enable_larger_pack(&mut self) {
        let cmd = protocol::build_handshake_cmd();
        // Send handshake. We don't strictly care if it fails, we just fallback.
        match self.usb_write_read(&cmd, 4) {
            Ok(resp) if !resp.is_empty() && resp[0] == protocol::CMD_JTAG_INIT => {
                debug!("CH347: Larger Pack mode enabled");
                self.larger_pack_supported = true;
            }
            _ => {
                debug!("CH347: Larger Pack mode not supported or handshake failed");
                self.larger_pack_supported = false;
            }
        }
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
        if self.larger_pack_supported {
            "CH347 High-Speed Programmer (Larger Pack)"
        } else {
            "CH347 High-Speed Programmer"
        }
    }

    fn spi_transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<()> {
        if tx.is_empty() {
            return Ok(());
        }

        let chunk_size = if self.larger_pack_supported {
            protocol::MAX_PACKET_SIZE_LARGER
        } else {
            protocol::MAX_PACKET_SIZE_STANDARD
        };

        for (tx_chunk, rx_chunk) in tx.chunks(chunk_size).zip(rx.chunks_mut(chunk_size)) {
            let packet = protocol::build_spi_transfer_cmd(tx_chunk);
            let response = self.usb_write_read(&packet, tx_chunk.len())?;

            let copy_len = response.len().min(rx_chunk.len());
            rx_chunk[..copy_len].copy_from_slice(&response[..copy_len]);

            if response.len() != tx_chunk.len() {
                return Err(Error::Other(format!(
                    "CH347: SPI transfer size mismatch (Sent {}, got {})",
                    tx_chunk.len(),
                    response.len()
                )));
            }
        }

        Ok(())
    }

    fn set_cs(&mut self, active: bool) -> Result<()> {
        let packet = protocol::build_cs_cmd(active);
        self.usb_write_read(&packet, 0)?;
        Ok(())
    }

    fn set_speed(&mut self, speed: u8) -> Result<()> {
        let spi_speed = protocol::SpiSpeed::from_u8(speed);
        debug!("CH347: Setting speed to {}", spi_speed.description());

        let packet = protocol::build_set_cfg_cmd(spi_speed);
        self.usb_write_read(&packet, 0)?;
        self.current_speed = speed;
        Ok(())
    }

    fn max_bulk_transfer_size(&self) -> usize {
        if self.larger_pack_supported {
            protocol::MAX_PACKET_SIZE_LARGER
        } else {
            protocol::MAX_PACKET_SIZE_STANDARD
        }
    }
}
