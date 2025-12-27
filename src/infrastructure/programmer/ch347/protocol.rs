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
pub const CMD_JTAG_INIT: u8 = 0xD0; // JTAG INIT (Also used for Larger Pack handshake)

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
///
/// Protocol Config Structure (Total 26 bytes payload + header):
/// Header: [0xC0, 0x1A, 0x00] (Command + Length)
/// Payload:
/// - Byte 9 (Payload[6]): Clock Polarity (Bit 1). 0=Low, 1=High
/// - Byte 11 (Payload[8]): Clock Phase (Bit 0). 0=First Edge, 1=Second Edge
/// - Byte 15 (Payload[12]): Clock Divisor (Bits 5:3). Values 0-7.
/// - Byte 17 (Payload[14]): Byte Order (Bit 7). 0=MSB First, 1=LSB First.
pub fn build_set_cfg_cmd(speed: SpiSpeed) -> Vec<u8> {
    let mut cfg = vec![0u8; 26]; // 26 bytes payload

    // 1. SPI Mode 0 (CPOL=0, CPHA=0)
    // Byte 9 (Payload 6): Clock Polarity (CPOL)
    cfg[6] = 0x00;
    // Byte 11 (Payload 8): Clock Phase (CPHA)
    cfg[8] = 0x00;

    // 2. SPI Speed
    // Byte 15 (Payload 12): Clock divisor (bits 5:3)
    // Map speed enum to divisor value
    cfg[12] = (speed as u8) << 3;

    // 3. Byte Order
    // Byte 17 (Payload 14): Bit 7 defines order. 0 = MSB First, 1 = LSB First.
    cfg[14] = 0x00; // MSB First

    let len = cfg.len();
    let mut packet = Vec::with_capacity(3 + len);
    packet.push(CMD_SPI_SET_CFG);
    packet.push((len & 0xFF) as u8);
    packet.push(((len >> 8) & 0xFF) as u8);
    packet.extend_from_slice(&cfg);
    packet
}

/// Build command for CS control
///
/// Protocol CS Structure (Total 10 bytes payload + header):
/// Header: [0xC1, 0x0A, 0x00] (Command + Length)
/// Payload:
/// - Byte 3 (Payload[0]): CS1 Control
///     - Bit 7: Mask (1=Update, 0=Ignore)
///     - Bit 6: Value (1=High/Inactive, 0=Low/Active)
/// - Byte 8 (Payload[5]): CS2 Control (Not used here, but supported)
pub fn build_cs_cmd(active: bool) -> Vec<u8> {
    let mut payload = vec![0u8; 10]; // Length 10

    // Byte 3 (Payload 0): CS1
    // Bit 7: Mask (1=Change)
    // Bit 6: Value (0=Assert/Low, 1=Deassert/High)
    let val_bit = if active { 0 } else { 1 };

    // Construct CS1 byte: Mask(0x80) | (Value << 6)
    payload[0] = 0x80 | (val_bit << 6);

    let len = payload.len();
    let mut packet = Vec::with_capacity(3 + len);
    packet.push(CMD_SPI_CONTROL);
    packet.push((len & 0xFF) as u8);
    packet.push(((len >> 8) & 0xFF) as u8);
    packet.extend_from_slice(&payload);
    packet
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

/// Build handshake command to check/enable Larger Pack mode
pub fn build_handshake_cmd() -> Vec<u8> {
    // [CMD, LenLow, LenHigh, ClockIndex, 0, 0, 0, 0, 0]
    // Clock Index 9 enables Larger Pack mode
    vec![
        CMD_JTAG_INIT,
        0x06,
        0x00,
        0x09,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
    ]
}

pub const MAX_PACKET_SIZE_STANDARD: usize = 510;
pub const MAX_PACKET_SIZE_LARGER: usize = 51184;
