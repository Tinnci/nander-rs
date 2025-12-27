//! FTDI-based Programmer Implementation
//!
//! Supports FT232H, FT2232H, and FT4232H in MPSSE SPI mode
//! VID: 0403 (FTDI), various PIDs

use crate::error::{Error, Result};
use crate::infrastructure::programmer::traits::Programmer;
use log::debug;

// FTDI USB identifiers
pub const FTDI_VID: u16 = 0x0403;
pub const FT232H_PID: u16 = 0x6014;
pub const FT2232H_PID: u16 = 0x6010;
pub const FT4232H_PID: u16 = 0x6011;

/// FTDI Programmer using MPSSE mode
pub struct FtdiProgrammer {
    // TODO: Add FTDI device handle
    // We'll need to choose between:
    // - libftdi1-sys (FFI bindings to libftdi)
    // - ftdi-embedded-hal (pure Rust, if mature enough)
    // - Direct libusb with manual MPSSE protocol
    current_speed: u32, // Speed in Hz
}

impl FtdiProgrammer {
    pub fn new(_device: nusb::Device) -> Result<Self> {
        debug!("Initializing FTDI programmer (MPSSE mode)");

        // TODO: Open FTDI device
        // TODO: Reset device
        // TODO: Configure MPSSE mode
        // TODO: Set initial SPI parameters

        Err(Error::NotSupported(
            "FTDI support is under development. Coming in v0.6.1!".to_string(),
        ))
    }
}

impl Programmer for FtdiProgrammer {
    fn name(&self) -> &str {
        "FTDI High-Speed Programmer (MPSSE)"
    }

    fn spi_transfer(&mut self, _tx: &[u8], _rx: &mut [u8]) -> Result<()> {
        // TODO: Implement MPSSE SPI transfer
        // MPSSE commands:
        // - 0x10: Clock Data Bytes Out on -ve clock edge MSB first (no read)
        // - 0x20: Clock Data Bytes In on +ve clock edge MSB first (no write)
        // - 0x31: Clock Data Bytes In and Out MSB first

        Err(Error::NotSupported(
            "FTDI SPI not yet implemented".to_string(),
        ))
    }

    fn set_cs(&mut self, _active: bool) -> Result<()> {
        // TODO: Use MPSSE GPIO commands to control CS
        // Typically ADBUS3 (DB3) is used for CS

        Err(Error::NotSupported(
            "FTDI CS control not yet implemented".to_string(),
        ))
    }

    fn set_speed(&mut self, speed: u8) -> Result<()> {
        // FTDI can support very high speeds:
        // FT232H: up to 30MHz
        // FT2232H: up to 30MHz (per channel)

        let speed_hz = match speed {
            0 => 100_000,    // 100kHz
            1 => 400_000,    // 400kHz
            2 => 1_000_000,  // 1MHz
            3 => 2_000_000,  // 2MHz
            4 => 5_000_000,  // 5MHz
            5 => 10_000_000, // 10MHz (default)
            6 => 20_000_000, // 20MHz
            7 => 30_000_000, // 30MHz (max)
            _ => 10_000_000,
        };

        self.current_speed = speed_hz;
        debug!("FTDI: Set speed to {} Hz", speed_hz);

        // TODO: Send MPSSE clock divisor command
        // Divisor = (60MHz / (2 * desired_freq)) - 1

        Ok(())
    }
}

// TODO: Helper functions for MPSSE protocol
// fn mpsse_clock_divisor(freq_hz: u32) -> u16 { ... }
// fn mpsse_set_gpio(pins: u8, direction: u8) -> Vec<u8> { ... }
