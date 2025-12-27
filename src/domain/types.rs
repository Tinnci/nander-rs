//! Domain Types - Core Business Types
//!
//! Common types used across the domain layer.

use std::fmt;

/// Flash memory capacity representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Capacity(pub u32);

impl Capacity {
    pub const fn bytes(bytes: u32) -> Self {
        Self(bytes)
    }

    pub const fn megabytes(mb: u32) -> Self {
        Self(mb * 1024 * 1024)
    }

    pub const fn gigabits(gb: u32) -> Self {
        Self(gb * 128 * 1024 * 1024) // 1 Gbit = 128 MiB
    }

    pub fn as_bytes(&self) -> u32 {
        self.0
    }

    pub fn as_megabytes(&self) -> u32 {
        self.0 / (1024 * 1024)
    }
}

impl fmt::Display for Capacity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mb = self.as_megabytes();
        if mb >= 1024 {
            write!(f, "{} GB", mb / 1024)
        } else if mb > 0 {
            write!(f, "{} MB", mb)
        } else {
            write!(f, "{} KB", self.0 / 1024)
        }
    }
}

/// Memory address
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Address(pub u32);

impl Address {
    pub fn new(addr: u32) -> Self {
        Self(addr)
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }

    pub fn page(&self, page_size: u32) -> u32 {
        self.0 / page_size
    }

    pub fn block(&self, block_size: u32) -> u32 {
        self.0 / block_size
    }
}

/// Flash type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlashType {
    /// SPI NAND Flash
    Nand,
    /// SPI NOR Flash
    Nor,
    /// SPI EEPROM (25xxx series)
    SpiEeprom,
    /// I2C EEPROM (24Cxx series)
    I2cEeprom,
    /// Microwire EEPROM (93Cxx series)
    MicrowireEeprom,
}

/// Common options for Flash operations
#[derive(Debug, Clone)]
pub struct FlashOptions {
    /// Starting address
    pub address: u32,
    /// Length of operation (if applicable)
    pub length: Option<u32>,
    /// Whether to use/check ECC
    pub use_ecc: bool,
    /// Whether to ignore ECC errors (useful for recovery)
    pub ignore_ecc_errors: bool,
    /// Strategy for handling NAND bad blocks
    pub bad_block_strategy: super::bad_block::BadBlockStrategy,
    /// How to handle NAND OOB data
    pub oob_mode: super::OobMode,
    /// SPI/I2C speed setting
    pub speed: Option<u8>,
    /// Whether to verify after write
    pub verify: bool,
    /// Number of retries for read operations
    pub retry_count: u32,
    /// Optional bad block table file path
    pub bbt_file: Option<std::path::PathBuf>,
}

impl Default for FlashOptions {
    fn default() -> Self {
        Self {
            address: 0,
            length: None,
            use_ecc: true,
            ignore_ecc_errors: false,
            bad_block_strategy: super::bad_block::BadBlockStrategy::Fail,
            oob_mode: super::OobMode::None,
            speed: None,
            verify: false,
            retry_count: 0,
            bbt_file: None,
        }
    }
}

impl fmt::Display for FlashType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlashType::Nand => write!(f, "SPI NAND"),
            FlashType::Nor => write!(f, "SPI NOR"),
            FlashType::SpiEeprom => write!(f, "SPI EEPROM"),
            FlashType::I2cEeprom => write!(f, "I2C EEPROM"),
            FlashType::MicrowireEeprom => write!(f, "Microwire EEPROM"),
        }
    }
}

/// JEDEC Manufacturer ID and Device ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JedecId {
    pub manufacturer: u8,
    pub device: u8,
    pub density: u8,
}

impl JedecId {
    pub fn new(data: [u8; 3]) -> Self {
        Self {
            manufacturer: data[0],
            device: data[1],
            density: data[2],
        }
    }

    pub fn as_bytes(&self) -> [u8; 3] {
        [self.manufacturer, self.device, self.density]
    }
}

impl fmt::Display for JedecId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02X} {:02X} {:02X}",
            self.manufacturer, self.device, self.density
        )
    }
}

/// Progress information for operations
#[derive(Debug, Clone)]
pub struct Progress {
    pub current: u64,
    pub total: u64,
    pub message: Option<String>,
}

impl Progress {
    pub fn new(current: u64, total: u64) -> Self {
        Self {
            current,
            total,
            message: None,
        }
    }

    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.current as f64 / self.total as f64) * 100.0
        }
    }
}
