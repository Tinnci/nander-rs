//! CH347 protocol definitions
//!
//! This module contains constants and helper functions for building
//! CH347 USB command packets.

// ============================================================================
// CH347 Command Bytes
// ============================================================================

pub const CMD_SPI_SET_CFG: u8 = 0xC0; // Configure SPI
pub const CMD_SPI_CONTROL: u8 = 0xC1; // CS control
pub const CMD_SPI_RD_WR: u8 = 0xC2; // Standard read/write
pub const CMD_SPI_BLCK_RD: u8 = 0xC3; // Block read
pub const CMD_SPI_BLCK_WR: u8 = 0xC4; // Block write

// ============================================================================
// CH347 SPI Speed Settings
// ============================================================================

/// SPI clock speed settings for CH347
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpiSpeed {
    /// 60 MHz
    Speed60M = 0,
    /// 30 MHz
    Speed30M = 1,
    /// 15 MHz
    Speed15M = 2,
    /// 7.5 MHz
    Speed7_5M = 3,
    /// 3.75 MHz
    Speed3_75M = 4,
    /// 1.875 MHz (default)
    #[default]
    Medium = 5,
    /// 937.5 KHz
    Speed937K = 6,
    /// 468.75 KHz
    Speed468K = 7,
}

impl SpiSpeed {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Speed60M,
            1 => Self::Speed30M,
            2 => Self::Speed15M,
            3 => Self::Speed7_5M,
            4 => Self::Speed3_75M,
            5 => Self::Medium,
            6 => Self::Speed937K,
            7 => Self::Speed468K,
            _ => Self::default(),
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Speed60M => "60 MHz",
            Self::Speed30M => "30 MHz",
            Self::Speed15M => "15 MHz",
            Self::Speed7_5M => "7.5 MHz",
            Self::Speed3_75M => "3.75 MHz",
            Self::Medium => "1.875 MHz",
            Self::Speed937K => "937.5 KHz",
            Self::Speed468K => "468.75 KHz",
        }
    }
}

// ============================================================================
// Command Builders
// ============================================================================

/// Build command to configure SPI settings
pub fn build_set_cfg_cmd(speed: SpiSpeed) -> Vec<u8> {
    let mut cfg = vec![0u8; 26];

    // Value mapping for iClock (Index 5)
    cfg[5] = speed as u8;

    // Mode (Index 1): SPI Mode 0 as default
    cfg[1] = 0;

    // Byte Order (Index 7): 1 = MSB First
    cfg[7] = 1;

    let len = cfg.len();
    let mut packet = Vec::with_capacity(3 + len);
    packet.push(CMD_SPI_SET_CFG);
    packet.push((len & 0xFF) as u8);
    packet.push(((len >> 8) & 0xFF) as u8);
    packet.extend_from_slice(&cfg);
    packet
}

/// Build command for CS control
pub fn build_cs_cmd(active: bool) -> Vec<u8> {
    // Usually 0 for active (low), 1 for inactive (high).
    let cs_level = if active { 0 } else { 1 };
    vec![CMD_SPI_CONTROL, 1, 0, cs_level]
}

/// Build command for SPI read/write transfer
pub fn build_spi_transfer_cmd(tx: &[u8]) -> Vec<u8> {
    let len = tx.len();
    let mut packet = Vec::with_capacity(3 + len);
    packet.push(CMD_SPI_RD_WR);
    packet.push((len & 0xFF) as u8);
    packet.push(((len >> 8) & 0xFF) as u8);
    packet.extend_from_slice(tx);
    packet
}
