//! CH340/CH340G/CH340K Serial Port Implementation
//!
//! The CH340 is a USB-to-UART bridge chip commonly used for Arduino and other
//! microcontroller programming. It's a simpler chip than CH341A, dedicated
//! specifically to serial communication.

use crate::error::{Error, Result};
use crate::infrastructure::programmer::traits::{Parity, SerialConfig, SerialPort, StopBits};
use futures_lite::future::block_on;
use log::debug;
use nusb::transfer::RequestBuffer;
use nusb::Device;

/// CH340 USB endpoints
const EP_OUT: u8 = 0x02;
const EP_IN: u8 = 0x82;

/// CH340 Control Request Codes
#[allow(dead_code)]
mod request {
    pub const READ_VERSION: u8 = 0x5F;
    pub const READ_REGISTRY: u8 = 0x95;
    pub const WRITE_REGISTRY: u8 = 0x9A;
    pub const SERIAL_INIT: u8 = 0xA1;
    pub const MODEM_CTRL: u8 = 0xA4;
}

/// CH340 Registry addresses
mod reg {
    pub const BAUD_FACTOR: u16 = 0x1312;
    pub const BAUD_OFFSET: u16 = 0x0F2C;
    pub const LCR1: u16 = 0x2518;
    pub const LCR2: u16 = 0x0C00;
}

/// CH340 Modem control bits
mod modem {
    pub const DTR: u8 = 0x20;
    pub const RTS: u8 = 0x40;
}

/// CH340/CH340G Serial Port
pub struct Ch340Serial {
    #[allow(dead_code)]
    device: Device,
    interface: nusb::Interface,
    config: SerialConfig,
    dtr: bool,
    rts: bool,
}

impl Ch340Serial {
    /// Create a new CH340 serial port instance
    pub fn new(device: Device) -> Result<Self> {
        let interface = device.claim_interface(0)?;

        let mut serial = Self {
            device,
            interface,
            config: SerialConfig::default(),
            dtr: false,
            rts: false,
        };

        // Initialize the chip
        serial.init()?;

        // Configure with default settings
        serial.configure(&SerialConfig::default())?;

        Ok(serial)
    }

    /// Initialize the CH340
    fn init(&mut self) -> Result<()> {
        debug!("Initializing CH340...");

        // Read version
        let mut version = [0u8; 2];
        self.control_in(request::READ_VERSION, 0, 0, &mut version)?;
        debug!("CH340 version: {:02X} {:02X}", version[0], version[1]);

        // Serial init
        self.control_out(request::SERIAL_INIT, 0, 0)?;

        Ok(())
    }

    /// Read modem status (CTS, DSR, RI, DCD lines)
    /// Returns (CTS, DSR, RI, DCD) as boolean tuple
    #[allow(dead_code)]
    pub fn read_modem_status(&self) -> Result<(bool, bool, bool, bool)> {
        let mut status = [0u8; 2];
        self.control_in(request::READ_REGISTRY, 0x0706, 0, &mut status)?;

        // CH340 modem status bits (active low):
        // Bit 0: CTS
        // Bit 1: DSR
        // Bit 2: RI
        // Bit 3: DCD
        let cts = (status[0] & 0x01) == 0;
        let dsr = (status[0] & 0x02) == 0;
        let ri = (status[0] & 0x04) == 0;
        let dcd = (status[0] & 0x08) == 0;

        Ok((cts, dsr, ri, dcd))
    }

    /// Calculate baud rate divisor for CH340
    fn calc_baud_divisor(baud: u32) -> (u16, u16) {
        // CH340 uses a 12MHz crystal
        // The baud rate formula is: baud = 12000000 / (factor * 2^(12 - offset))
        // We need to find factor and offset that give us the closest baud rate

        // Common pre-calculated values
        let (factor, offset) = match baud {
            300 => (0x2710, 0x00),
            600 => (0x2710, 0x01),
            1200 => (0x2710, 0x02),
            2400 => (0x2710, 0x03),
            4800 => (0x2710, 0x04),
            9600 => (0x2710, 0x05),
            14400 => (0x0D98, 0x07),
            19200 => (0x2710, 0x06),
            38400 => (0x2710, 0x07),
            57600 => (0x1A00, 0x08),
            115200 => (0x1A00, 0x09),
            230400 => (0x1A00, 0x0A),
            460800 => (0x1A00, 0x0B),
            921600 => (0x1A00, 0x0C),
            1000000 => (0x180F, 0x0C),
            2000000 => (0x180F, 0x0D),
            3000000 => (0x180F, 0x0E),
            _ => {
                // Generic calculation for custom baud rates
                let divisor = 12_000_000 / baud;
                if divisor <= 0x2710 {
                    (divisor as u16, 0x0C)
                } else {
                    (0x2710, 0x05) // fallback to 9600
                }
            }
        };

        (factor, offset)
    }

    /// Control transfer OUT (host to device)
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

    /// Control transfer IN (device to host)
    fn control_in(&self, request: u8, value: u16, index: u16, buf: &mut [u8]) -> Result<usize> {
        let result = block_on(async {
            self.interface
                .control_in(nusb::transfer::ControlIn {
                    control_type: nusb::transfer::ControlType::Vendor,
                    recipient: nusb::transfer::Recipient::Device,
                    request,
                    value,
                    index,
                    length: buf.len() as u16,
                })
                .await
                .into_result()
        })
        .map_err(|e| Error::Other(e.to_string()))?;
        let len = result.len().min(buf.len());
        buf[..len].copy_from_slice(&result[..len]);
        Ok(len)
    }

    /// Update modem control lines (DTR/RTS)
    fn update_modem_ctrl(&self) -> Result<()> {
        let mut ctrl = 0u8;
        if self.dtr {
            ctrl |= modem::DTR;
        }
        if self.rts {
            ctrl |= modem::RTS;
        }
        // CH340 inverts the logic
        ctrl = !ctrl;
        self.control_out(request::MODEM_CTRL, ctrl as u16, 0)?;
        Ok(())
    }
}

impl SerialPort for Ch340Serial {
    fn name(&self) -> &str {
        "CH340 Serial Port"
    }

    fn configure(&mut self, config: &SerialConfig) -> Result<()> {
        debug!(
            "Configuring CH340: {} baud, {}N{} (simplified)",
            config.baud_rate,
            config.data_bits,
            config.stop_bits.as_str()
        );

        self.config = config.clone();

        // Set baud rate
        let (factor, offset) = Self::calc_baud_divisor(config.baud_rate);
        self.control_out(request::WRITE_REGISTRY, reg::BAUD_FACTOR, factor)?;
        self.control_out(request::WRITE_REGISTRY, reg::BAUD_OFFSET, offset)?;

        // Set line control (data bits, parity, stop bits)
        let mut lcr: u16 = 0;

        // Data bits
        lcr |= match config.data_bits {
            5 => 0x00,
            6 => 0x01,
            7 => 0x02,
            8 => 0x03,
            _ => 0x03, // default 8
        };

        // Stop bits
        lcr |= match config.stop_bits {
            StopBits::One => 0x00,
            StopBits::OnePointFive => 0x04,
            StopBits::Two => 0x04,
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
        use std::time::Duration;

        // Use a short timeout to keep the worker responsive
        let read_future = self
            .interface
            .bulk_in(EP_IN, RequestBuffer::new(buffer.len().min(64)));

        let timeout_future = async {
            async_io::Timer::after(Duration::from_millis(10)).await;
        };

        let result = block_on(async {
            futures_lite::future::or(
                async {
                    let r = read_future.await;
                    Some(r)
                },
                async {
                    timeout_future.await;
                    None
                },
            )
            .await
        });

        match result {
            Some(transfer_result) => match transfer_result.into_result() {
                Ok(data) => {
                    let len = data.len().min(buffer.len());
                    buffer[..len].copy_from_slice(&data[..len]);
                    Ok(len)
                }
                Err(e) => Err(Error::Other(format!("USB Error: {}", e))),
            },
            None => Ok(0), // Timeout - no data available
        }
    }

    fn write(&mut self, data: &[u8]) -> Result<usize> {
        let result = block_on(self.interface.bulk_out(EP_OUT, data.to_vec()));
        result.status.map_err(|e| Error::Other(e.to_string()))?;
        Ok(data.len())
    }

    fn set_dtr(&mut self, level: bool) -> Result<()> {
        self.dtr = level;
        self.update_modem_ctrl()
    }

    fn set_rts(&mut self, level: bool) -> Result<()> {
        self.rts = level;
        self.update_modem_ctrl()
    }

    fn get_dtr(&self) -> Option<bool> {
        Some(self.dtr)
    }

    fn get_rts(&self) -> Option<bool> {
        Some(self.rts)
    }

    fn flush(&mut self) -> Result<()> {
        // CH340 doesn't have an explicit flush command
        // Data is sent immediately on bulk_out
        Ok(())
    }

    fn bytes_available(&self) -> Result<usize> {
        // CH340 doesn't provide a way to query available bytes
        // The read function handles this with non-blocking behavior
        Ok(0)
    }
}
