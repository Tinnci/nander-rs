//! Raspberry Pi / Linux spidev Programmer Implementation
//!
//! Uses native Linux SPI interface (/dev/spidevX.Y)
//! Requires spidev enabled in device tree

use crate::error::{Error, Result};
use crate::infrastructure::programmer::traits::Programmer;
use log::debug;

/// Linux spidev-based Programmer
pub struct SpidevProgrammer {
    // TODO: Add spidev file handle
    // We can use the `spidev` crate from crates.io
    #[allow(dead_code)]
    device_path: String,
    #[allow(dead_code)]
    cs_gpio_pin: Option<u32>, // GPIO pin for manual CS control
    current_speed: u32,
}

impl SpidevProgrammer {
    /// Create a new spidev programmer
    ///
    /// # Arguments
    /// * `device_path` - Path to spidev device (e.g., "/dev/spidev0.0")
    /// * `cs_pin` - Optional GPIO pin number for manual CS control
    pub fn new(device_path: String, _cs_pin: Option<u32>) -> Result<Self> {
        debug!("Initializing spidev programmer: {}", device_path);

        // TODO: Open spidev device
        // TODO: Configure SPI mode (CPOL=0, CPHA=0 for Mode 0)
        // TODO: Set initial speed
        // TODO: Set bits per word (8)

        // For GPIO CS control:
        // TODO: Export GPIO pin via /sys/class/gpio
        // TODO: Set GPIO direction to output

        Err(Error::NotSupported(
            "spidev support is under development. Coming in v0.6.1!".to_string(),
        ))
    }

    /// Create from default Raspberry Pi SPI0 CE0
    pub fn new_raspberry_pi_default() -> Result<Self> {
        Self::new("/dev/spidev0.0".to_string(), None)
    }
}

impl Programmer for SpidevProgrammer {
    fn name(&self) -> &str {
        "Linux spidev Programmer"
    }

    fn spi_transfer(&mut self, _tx: &[u8], _rx: &mut [u8]) -> Result<()> {
        // TODO: Use ioctl SPI_IOC_MESSAGE for full-duplex transfer
        // Or use the `spidev` crate's transfer() method

        Err(Error::NotSupported(
            "spidev SPI not yet implemented".to_string(),
        ))
    }

    fn set_cs(&mut self, _active: bool) -> Result<()> {
        // If cs_gpio_pin is Some, control GPIO manually
        // Otherwise, let kernel handle CS automatically

        // TODO: Write to /sys/class/gpio/gpioN/value

        Ok(()) // Kernel CS is automatic
    }

    fn set_speed(&mut self, speed: u8) -> Result<()> {
        let speed_hz = match speed {
            0 => 100_000,    // 100kHz
            1 => 500_000,    // 500kHz
            2 => 1_000_000,  // 1MHz
            3 => 2_000_000,  // 2MHz
            4 => 5_000_000,  // 5MHz
            5 => 10_000_000, // 10MHz (default)
            6 => 20_000_000, // 20MHz
            7 => 50_000_000, // 50MHz (Raspberry Pi 4 max)
            _ => 10_000_000,
        };

        self.current_speed = speed_hz;
        debug!("spidev: Set speed to {} Hz", speed_hz);

        // TODO: Use ioctl SPI_IOC_WR_MAX_SPEED_HZ

        Ok(())
    }
}

// TODO: GPIO helper functions
// fn gpio_export(pin: u32) -> Result<()> { ... }
// fn gpio_set_direction(pin: u32, direction: &str) -> Result<()> { ... }
// fn gpio_write(pin: u32, value: bool) -> Result<()> { ... }
