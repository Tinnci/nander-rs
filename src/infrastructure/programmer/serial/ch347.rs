//! CH347 Serial Port Implementation
//!
//! CH347 is a high-speed USB-to-UART/SPI/I2C/JTAG chip.
//! In Mode 1 (UART + SPI + I2C), UART is typically available on Interface 0.

use crate::error::{Error, Result};
use crate::infrastructure::programmer::traits::{Parity, SerialConfig, SerialPort, StopBits};
use futures_lite::future::block_on;
use log::debug;
use nusb::transfer::RequestBuffer;
use nusb::Device;

/// CH347 UART Endpoints (Standard CDC or Vendor-specific)
/// In Mode 1/2, CH347 usually has UART on Interface 0.
const EP_OUT: u8 = 0x02;
const EP_IN: u8 = 0x82;

/// CH347 Control Request Codes (Similar to CH340 for configuration)
mod request {
    pub const WRITE_REGISTRY: u8 = 0x9A;
}

/// CH347 Registry addresses
mod reg {
    pub const BAUD_FACTOR: u16 = 0x1312;
    pub const BAUD_OFFSET: u16 = 0x0F2C;
    pub const LCR1: u16 = 0x2518;
    pub const LCR2: u16 = 0x0C00;
}

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

    /// Calculate baud rate divisor (Uses same logic as CH340)
    fn calc_baud_divisor(baud: u32) -> (u16, u16) {
        // Common pre-calculated values for CH34x Series
        match baud {
            300 => (0x2710, 0x00),
            600 => (0x2710, 0x01),
            1200 => (0x2710, 0x02),
            2400 => (0x2710, 0x03),
            4800 => (0x2710, 0x04),
            9600 => (0x2710, 0x05),
            19200 => (0x2710, 0x06),
            38400 => (0x2710, 0x07),
            57600 => (0x1A00, 0x08),
            115200 => (0x1A00, 0x09),
            230400 => (0x1A00, 0x0A),
            460800 => (0x1A00, 0x0B),
            921600 => (0x1A00, 0x0C),
            _ => (0x1A00, 0x09), // default 115200
        }
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

        // 1. Set baud rate
        let (factor, offset) = Self::calc_baud_divisor(config.baud_rate);
        self.control_out(request::WRITE_REGISTRY, reg::BAUD_FACTOR, factor)?;
        self.control_out(request::WRITE_REGISTRY, reg::BAUD_OFFSET, offset)?;

        // 2. Set line control (data bits, parity, stop bits)
        let mut lcr: u16 = 0;

        // Data bits
        lcr |= match config.data_bits {
            5 => 0x00,
            6 => 0x01,
            7 => 0x02,
            8 => 0x03,
            _ => 0x03, // Default to 8 bits
        };

        // Stop bits
        lcr |= match config.stop_bits {
            StopBits::One => 0x00,
            _ => 0x04, // 1.5 or 2 stop bits
        };

        // Parity
        lcr |= match config.parity {
            Parity::None => 0x00,
            Parity::Odd => 0x08,
            Parity::Even => 0x18,
            Parity::Mark => 0x28,
            Parity::Space => 0x38,
        };

        self.control_out(request::WRITE_REGISTRY, reg::LCR1, lcr)?;
        self.control_out(request::WRITE_REGISTRY, reg::LCR2, 0)?;

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
            Err(e) => Err(Error::Other(format!("USB Error: {}", e))),
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
