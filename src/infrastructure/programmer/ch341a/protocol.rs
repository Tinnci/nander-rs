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

/// Get Status / Input bits
pub const CMD_GET_STATUS: u8 = 0xA1;

/// I2C subcommands for CMD_I2C_STREAM
pub mod i2c_sub {
    /// Start bit
    pub const START: u8 = 0x74;
    /// Stop bit
    pub const STOP: u8 = 0x75;
    /// Output data (write). OR with length (0-31)
    pub const OUT: u8 = 0x80;
    /// Input data (read). OR with length (0-31)
    pub const IN: u8 = 0xC0;
    /// Set speed/MS. OR with 0-7?
    pub const SET: u8 = 0x60;
    /// End of stream
    pub const END: u8 = 0x00;
}

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
    /// D0 - SPI CS / Microwire CS (Output)
    pub const CS: u8 = 0;
    /// D1 - SPI CLK / Microwire SK (Output)
    pub const CLK: u8 = 1;
    /// D2 - SPI DIN（MISO）/ Microwire DO (Input)
    pub const DIN: u8 = 2;
    /// D3 - SPI DOUT (MOSI) / Microwire DI (Output)
    pub const DOUT: u8 = 3;
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
        CMD_UIO_STM_OUT | 0x37, // Set all output pins high initially (CS high)
        CMD_UIO_STM_DIR | 0x3B, // Dir: D0, D1, D3, D4, D5 as Out (111011b = 0x3B)
        CMD_UIO_STM_US | (speed as u8),
        CMD_UIO_STM_END,
    ]
}

/// Build command to set all outputs
pub fn build_uio_out_cmd(outputs: u8) -> Vec<u8> {
    vec![
        CMD_UIO_STREAM,
        CMD_UIO_STM_OUT | (outputs & 0x3F),
        CMD_UIO_STM_END,
    ]
}

/// Build command to set CS state
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
pub fn build_spi_transfer_cmd(data: &[u8]) -> Vec<u8> {
    let mut cmd = Vec::with_capacity(data.len() + 1);
    cmd.push(CMD_SPI_STREAM);
    cmd.extend_from_slice(data);
    cmd
}

/// Build GPIO control command
pub fn build_gpio_cmd(pin: u8, level: bool, current_outputs: u8) -> Vec<u8> {
    let mask = 1u8 << pin;
    let new_outputs = if level {
        current_outputs | mask
    } else {
        current_outputs & !mask
    };

    vec![
        CMD_UIO_STREAM,
        CMD_UIO_STM_OUT | (new_outputs & 0x3F),
        CMD_UIO_STM_END,
    ]
}

/// Maximum SPI transfer size in one USB packet (standard mode)
pub const MAX_SPI_TRANSFER_SIZE: usize = 32;

/// Maximum USB bulk transfer size (CH341A hardware limit)
pub const MAX_USB_BULK_SIZE: usize = 4096;

/// Maximum SPI stream size per USB transaction
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
pub fn build_spi_stream_cmd(len: usize) -> Vec<u8> {
    let mut cmd = Vec::with_capacity(len + 1);
    cmd.push(CMD_SPI_STREAM);
    cmd.resize(len + 1, 0xFF);
    cmd
}
