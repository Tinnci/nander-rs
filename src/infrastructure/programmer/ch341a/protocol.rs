//! CH341A protocol definitions
//!
//! This module contains constants and helper functions for building
//! CH341A USB command packets.
//!
//! Reference: CH341A datasheet and libusb-based implementations

// ============================================================================
// CH341A Command Bytes
// ============================================================================

/// Stream command mode
pub const CMD_SPI_STREAM: u8 = 0xA8;

/// Set I/O stream mode
pub const CMD_I2C_STREAM: u8 = 0xAA;

/// UIO stream command (bit-bang mode)
pub const CMD_UIO_STREAM: u8 = 0xAB;

/// I2C/SPI command stream end
pub const CMD_I2C_STM_END: u8 = 0x00;

/// Set CS (chip select) state
pub const CMD_UIO_STM_OUT: u8 = 0x80;

/// Set direction
pub const CMD_UIO_STM_DIR: u8 = 0x40;

/// End of UIO command
pub const CMD_UIO_STM_END: u8 = 0x20;

/// Set SPI speed
pub const CMD_UIO_STM_US: u8 = 0xC0;

// ============================================================================
// CH341A SPI Speed Settings
// ============================================================================

/// SPI clock speed settings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpiSpeed {
    /// ~20.8 kHz
    Speed208K = 0,
    /// ~100 kHz
    Speed100K = 1,
    /// ~400 kHz
    Speed400K = 2,
    /// ~750 kHz
    Speed750K = 3,
    /// ~1.5 MHz
    Speed1_5M = 4,
    /// ~3 MHz (default, most compatible)
    #[default]
    Medium = 5,
    /// ~6 MHz
    Speed6M = 6,
    /// ~12 MHz
    Speed12M = 7,
}

impl SpiSpeed {
    /// Create SpiSpeed from a u8 value (0-7)
    /// Returns default (Medium/3MHz) for invalid values
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Speed208K,
            1 => Self::Speed100K,
            2 => Self::Speed400K,
            3 => Self::Speed750K,
            4 => Self::Speed1_5M,
            5 => Self::Medium,
            6 => Self::Speed6M,
            7 => Self::Speed12M,
            _ => Self::default(),
        }
    }

    /// Get human-readable speed description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Speed208K => "~21 kHz",
            Self::Speed100K => "~100 kHz",
            Self::Speed400K => "~400 kHz",
            Self::Speed750K => "~750 kHz",
            Self::Speed1_5M => "~1.5 MHz",
            Self::Medium => "~3 MHz",
            Self::Speed6M => "~6 MHz",
            Self::Speed12M => "~12 MHz",
        }
    }
}

// ============================================================================
// Pin Definitions
// ============================================================================

/// CH341A output pin assignments (directly from CH341A pinout)
pub mod pins {
    /// D0 - SPI CS (directly controlled)
    pub const CS: u8 = 0;
    /// D1 - SPI CLK (directly controlled)
    pub const CLK: u8 = 1;
    /// D2 - SPI MISO (directly controlled)
    pub const MISO: u8 = 2;
    /// D3 - SPI MOSI (directly controlled)
    pub const MOSI: u8 = 3;
    /// D4 - General purpose output (can control WP)
    pub const D4: u8 = 4;
    /// D5 - General purpose output (can control HOLD)
    pub const D5: u8 = 5;
}

// ============================================================================
// Command Builders
// ============================================================================

/// Build command to set SPI mode and speed
pub fn build_set_mode_cmd(speed: SpiSpeed) -> Vec<u8> {
    vec![
        CMD_UIO_STREAM,
        CMD_UIO_STM_OUT | 0x37, // Set all output pins high initially
        CMD_UIO_STM_DIR | 0x3F, // Set direction (0-5 as outputs)
        CMD_UIO_STM_US | (speed as u8),
        CMD_UIO_STM_END,
    ]
}

/// Build command to set chip select state
///
/// # Arguments
/// * `active` - true = CS low (active), false = CS high (inactive)
pub fn build_cs_cmd(active: bool) -> Vec<u8> {
    let output_byte = if active {
        0x36 // CS low (bit 0 = 0), others high
    } else {
        0x37 // CS high (bit 0 = 1), others high
    };

    vec![
        CMD_UIO_STREAM,
        CMD_UIO_STM_OUT | output_byte,
        CMD_UIO_STM_END,
    ]
}

/// Build SPI transfer command
///
/// This wraps the data in a SPI stream packet for the CH341A.
pub fn build_spi_transfer_cmd(data: &[u8]) -> Vec<u8> {
    let mut cmd = Vec::with_capacity(data.len() + 1);
    cmd.push(CMD_SPI_STREAM);
    cmd.extend_from_slice(data);
    cmd
}

/// Build GPIO control command
///
/// # Arguments
/// * `pin` - Pin number (4 or 5 for D4/D5)
/// * `level` - true = high, false = low
pub fn build_gpio_cmd(pin: u8, level: bool) -> Vec<u8> {
    // Pins D4 and D5 are controllable as GPIO
    let mask = 1u8 << pin;
    let output_byte = if level {
        0x37 | mask // Set the pin high
    } else {
        0x37 & !mask // Set the pin low
    };

    vec![
        CMD_UIO_STREAM,
        CMD_UIO_STM_OUT | output_byte,
        CMD_UIO_STM_END,
    ]
}

/// Maximum SPI transfer size in one USB packet (standard mode)
pub const MAX_SPI_TRANSFER_SIZE: usize = 32;

/// Maximum USB bulk transfer size (CH341A hardware limit)
/// The CH341A can handle larger USB packets, we use 4KB for optimal throughput
pub const MAX_USB_BULK_SIZE: usize = 4096;

/// Maximum SPI stream size per USB transaction
/// Account for command byte overhead
pub const MAX_SPI_STREAM_SIZE: usize = MAX_USB_BULK_SIZE - 1;

// ============================================================================
// Packet Helpers
// ============================================================================

/// Split a large transfer into CH341A-compatible chunks
pub fn chunk_transfer(data: &[u8]) -> impl Iterator<Item = &[u8]> {
    data.chunks(MAX_SPI_TRANSFER_SIZE)
}

/// Split a large transfer into bulk-optimized chunks
pub fn chunk_transfer_bulk(data: &[u8]) -> impl Iterator<Item = &[u8]> {
    data.chunks(MAX_SPI_STREAM_SIZE)
}

/// Build a bulk SPI stream command for larger transfers
///
/// This allows reading more data in a single USB transaction
pub fn build_spi_stream_cmd(len: usize) -> Vec<u8> {
    // For reading, we send 0xFF bytes which the flash ignores
    let mut cmd = Vec::with_capacity(len + 1);
    cmd.push(CMD_SPI_STREAM);
    cmd.resize(len + 1, 0xFF);
    cmd
}
