//! CH347 Serial Port Implementation
//!
//! CH347 is a high-speed USB-to-UART/SPI/I2C/JTAG chip.
//! In Mode 1 (UART + SPI + I2C), UART is typically available on Interface 0.

use crate::error::{Error, Result};
use crate::infrastructure::programmer::traits::{SerialConfig, SerialPort};
use futures_lite::future::block_on;
use log::debug;
use nusb::transfer::RequestBuffer;
use nusb::Device;

/// CH347 UART Endpoints (Standard CDC or Vendor-specific)
/// In Mode 1/2, CH347 usually has UART on Interface 0.
const EP_OUT: u8 = 0x02;
const EP_IN: u8 = 0x82;

/// CH347 Serial Implementation
pub struct Ch347Serial {
    #[allow(dead_code)]
    device: Device,
    interface: nusb::Interface,
    config: SerialConfig,
}

impl Ch347Serial {
    /// Create a new CH347 serial port instance
    pub fn new(device: Device) -> Result<Self> {
        // Use Interface 0 for UART
        let interface = device
            .claim_interface(0)
            .map_err(|e| Error::Other(format!("Failed to claim CH347 UART interface: {}", e)))?;

        let mut serial = Self {
            device,
            interface,
            config: SerialConfig::default(),
        };

        // Initialize with default settings
        serial.configure(&SerialConfig::default())?;

        Ok(serial)
    }

    /// Control transfer for CH347 UART configuration
    #[allow(dead_code)]
    fn control_out(&self, request: u8, value: u16, index: u16) -> Result<()> {
        let result = block_on(async {
            self.interface
                .control_out(nusb::transfer::ControlOut {
                    control_type: nusb::transfer::ControlType::Vendor,
                    recipient: nusb::transfer::Recipient::Device,
                    request,
                    value,
                    index,
                    data: &[],
                })
                .await
        });
        result.status.map_err(|e| Error::Other(e.to_string()))?;
        Ok(())
    }
}

impl SerialPort for Ch347Serial {
    fn name(&self) -> &str {
        "CH347 UART"
    }

    fn configure(&mut self, config: &SerialConfig) -> Result<()> {
        debug!(
            "Configuring CH347 UART: {} baud, {}N{}",
            config.baud_rate,
            config.data_bits,
            config.stop_bits.as_str()
        );

        self.config = config.clone();

        // CH347 UART configuration is complex.
        // For now, we'll try to use a simplified approach or assume it's already in a good state.
        // TODO: Implement full CH347 UART configuration (Baud rate, etc.)
        // CH347 actually uses Interface 0 as a standard CDC-ACM or Vendor UART.
        // If it's VCP (Virtual COM Port) mode, it might need specific control transfers.

        Ok(())
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let result = block_on(
            self.interface
                .bulk_in(EP_IN, RequestBuffer::new(buffer.len().min(512))),
        );

        match result.into_result() {
            Ok(data) => {
                let len = data.len().min(buffer.len());
                buffer[..len].copy_from_slice(&data[..len]);
                Ok(len)
            }
            Err(_) => Ok(0),
        }
    }

    fn write(&mut self, data: &[u8]) -> Result<usize> {
        let result = block_on(self.interface.bulk_out(EP_OUT, data.to_vec()));
        result.status.map_err(|e| Error::Other(e.to_string()))?;
        Ok(data.len())
    }

    fn set_dtr(&mut self, _level: bool) -> Result<()> {
        // TODO: Implement DTR control for CH347
        Ok(())
    }

    fn set_rts(&mut self, _level: bool) -> Result<()> {
        // TODO: Implement RTS control for CH347
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }

    fn bytes_available(&self) -> Result<usize> {
        Ok(0)
    }
}
