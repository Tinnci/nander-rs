//! CH347 protocol definitions
//!
//! This module contains constants and packet builders for communicating
//! with WCH CH347 (CH347T and CH347F) in SPI/I2C mode over USB.

// ============================================================================
// CH347 Command Bytes
// ============================================================================

pub const CMD_SPI_SET_CFG: u8 = 0xC0; // Configure SPI
pub const CMD_SPI_CONTROL: u8 = 0xC1; // CS control
pub const CMD_SPI_RD_WR: u8 = 0xC2; // Standard read/write (full duplex)
pub const CMD_SPI_BLCK_RD: u8 = 0xC3; // Block read
pub const CMD_SPI_BLCK_WR: u8 = 0xC4; // Block write
pub const CMD_SPI_GET_CFG: u8 = 0xCA; // Query SPI config
pub const CMD_JTAG_INIT: u8 = 0xD0; // JTAG init
pub const CMD_I2C_STREAM: u8 = 0xAA; // I2C stream command
pub const CMD_CH347F_SWITCH: u8 = 0xE2; // CH347F mode switch

// ============================================================================
// CH347 Buffer & Packet Limits
// ============================================================================

pub const CH347_PACKET_SIZE: usize = 510;
pub const CH347_MAX_DATA_LEN: usize = 507; // 510 - 3 bytes header

// Default USB endpoints & interfaces for CH347 SPI/I2C
pub const DEFAULT_WRITE_EP: u8 = 0x06;
pub const DEFAULT_READ_EP: u8 = 0x86;
pub const MODE_1_INTERFACE: u8 = 2; // Vendor-specific Interface 2 for SPI/I2C in Mode 1
pub const MODE_2_INTERFACE: u8 = 1; // Interface 1 in Mode 2 (HID)

// ============================================================================
// I2C Sub-commands
// ============================================================================

pub mod i2c_sub {
    pub const START: u8 = 0x74;
    pub const STOP: u8 = 0x75;
    pub const OUT: u8 = 0x80;
    pub const IN: u8 = 0xC0;
    pub const SET: u8 = 0x63;
    pub const END: u8 = 0x00;
}

// ============================================================================
// CH347 SPI Speed Settings
// ============================================================================

/// SPI clock speed settings for CH347
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpiSpeed {
    /// 60 MHz (default in SNANDer)
    #[default]
    Speed60M = 0,
    /// 30 MHz
    Speed30M = 1,
    /// 15 MHz
    Speed15M = 2,
    /// 7.5 MHz
    Speed7_5M = 3,
    /// 3.75 MHz
    Speed3_75M = 4,
    /// 1.875 MHz
    Speed1_875M = 5,
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
            5 => Self::Speed1_875M,
            6 => Self::Speed937K,
            7 => Self::Speed468K,
            _ => Self::default(),
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Speed60M => "60 MHz",
            Self::Speed30M => "30 MHz",
            Self::Speed15M => "15 MHz",
            Self::Speed7_5M => "7.5 MHz",
            Self::Speed3_75M => "3.75 MHz",
            Self::Speed1_875M => "1.875 MHz",
            Self::Speed937K => "937.5 KHz",
            Self::Speed468K => "468.75 KHz",
        }
    }
}

// ============================================================================
// Command Builders
// ============================================================================

/// Build 29-byte command to configure CH347 SPI settings
/// Matches SNANDer / WCH driver config_stream:
/// - Byte 0: CMD (0xC0)
/// - Byte 1-2: Payload length (26, little endian)
/// - Byte 5: 4
/// - Byte 6: 1
/// - Byte 9: Clock polarity (CPOL, 0 = Low)
/// - Byte 11: Clock phase (CPHA, 0 = First edge)
/// - Byte 14: 2
/// - Byte 15: Clock divisor bits 5:3
/// - Byte 17: Bit order (0 = MSB first)
/// - Byte 19: 7
/// - Byte 24: CS polarity (0 = active low)
pub fn build_set_cfg_cmd(speed: SpiSpeed) -> Vec<u8> {
    let mut cfg = vec![0u8; 29];
    cfg[0] = CMD_SPI_SET_CFG;
    cfg[1] = 26;
    cfg[2] = 0;
    cfg[5] = 4;
    cfg[6] = 1;
    cfg[9] = 0;
    cfg[11] = 0;
    cfg[14] = 2;
    cfg[15] = (speed.to_u8() & 0x07) << 3;
    cfg[17] = 0;
    cfg[19] = 7;
    cfg[24] = 0;
    cfg
}

/// Build 13-byte command for CS control
/// Matches SNANDer ch347_cs_control:
/// - Byte 0: CMD (0xC1)
/// - Byte 1-2: Payload length (10, little endian)
/// - Byte 3: CS1 Control (0x80: Assert/Low, 0xC0: Deassert/High)
/// - Byte 8: CS2 Control (0: Ignore)
pub fn build_cs_cmd(active: bool) -> Vec<u8> {
    let cs1 = if active { 0x80 } else { 0xC0 };
    let mut cmd = vec![0u8; 13];
    cmd[0] = CMD_SPI_CONTROL;
    cmd[1] = 10;
    cmd[2] = 0;
    cmd[3] = cs1;
    cmd[8] = 0;
    cmd
}

/// Build packet for SPI block write (0xC4)
/// Used for transmit-only commands and programming data
pub fn build_spi_write_packet(data_chunk: &[u8]) -> Vec<u8> {
    let len = data_chunk.len();
    let mut packet = Vec::with_capacity(3 + len);
    packet.push(CMD_SPI_BLCK_WR);
    packet.push((len & 0xFF) as u8);
    packet.push(((len >> 8) & 0xFF) as u8);
    packet.extend_from_slice(data_chunk);
    packet
}

/// Build command for SPI block read (0xC3)
/// Matches SNANDer ch347_read: sends 7 bytes containing 32-bit read count
pub fn build_spi_read_cmd(read_len: usize) -> Vec<u8> {
    vec![
        CMD_SPI_BLCK_RD,
        4,
        0,
        (read_len & 0xFF) as u8,
        ((read_len >> 8) & 0xFF) as u8,
        ((read_len >> 16) & 0xFF) as u8,
        ((read_len >> 24) & 0xFF) as u8,
    ]
}

/// Build packet for full-duplex SPI transfer (0xC2)
pub fn build_spi_transfer_packet(tx: &[u8]) -> Vec<u8> {
    let len = tx.len();
    let mut packet = Vec::with_capacity(3 + len);
    packet.push(CMD_SPI_RD_WR);
    packet.push((len & 0xFF) as u8);
    packet.push(((len >> 8) & 0xFF) as u8);
    packet.extend_from_slice(tx);
    packet
}

/// Build command to query SPI configuration (0xCA)
pub fn build_get_cfg_cmd() -> Vec<u8> {
    vec![CMD_SPI_GET_CFG, 0, 0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_cfg_packet() {
        let cmd = build_set_cfg_cmd(SpiSpeed::Speed60M);
        assert_eq!(cmd.len(), 29);
        assert_eq!(cmd[0], CMD_SPI_SET_CFG);
        assert_eq!(cmd[1], 26);
        assert_eq!(cmd[2], 0);
        assert_eq!(cmd[5], 4);
        assert_eq!(cmd[6], 1);
        assert_eq!(cmd[14], 2);
        assert_eq!(cmd[15], 0 << 3); // Speed 0
        assert_eq!(cmd[19], 7);
        assert_eq!(cmd[24], 0); // Active low

        let cmd30 = build_set_cfg_cmd(SpiSpeed::Speed30M);
        assert_eq!(cmd30[15], 1 << 3);
    }

    #[test]
    fn test_cs_packet() {
        let cs_assert = build_cs_cmd(true);
        assert_eq!(cs_assert.len(), 13);
        assert_eq!(cs_assert[0], CMD_SPI_CONTROL);
        assert_eq!(cs_assert[1], 10);
        assert_eq!(cs_assert[3], 0x80); // Assert

        let cs_deassert = build_cs_cmd(false);
        assert_eq!(cs_deassert.len(), 13);
        assert_eq!(cs_deassert[0], CMD_SPI_CONTROL);
        assert_eq!(cs_deassert[3], 0xC0); // Deassert
    }

    #[test]
    fn test_spi_write_packet() {
        let data = [0x9F, 0x00, 0x01];
        let pkt = build_spi_write_packet(&data);
        assert_eq!(pkt.len(), 6);
        assert_eq!(pkt[0], CMD_SPI_BLCK_WR);
        assert_eq!(pkt[1], 3);
        assert_eq!(pkt[2], 0);
        assert_eq!(&pkt[3..], &data);
    }

    #[test]
    fn test_spi_read_cmd() {
        let cmd = build_spi_read_cmd(2048);
        assert_eq!(cmd.len(), 7);
        assert_eq!(cmd[0], CMD_SPI_BLCK_RD);
        assert_eq!(cmd[1], 4);
        assert_eq!(cmd[2], 0);
        assert_eq!(cmd[3], 0x00);
        assert_eq!(cmd[4], 0x08);
        assert_eq!(cmd[5], 0x00);
        assert_eq!(cmd[6], 0x00);
    }

    #[test]
    fn test_spi_transfer_packet() {
        let data = [0x05, 0x00];
        let pkt = build_spi_transfer_packet(&data);
        assert_eq!(pkt.len(), 5);
        assert_eq!(pkt[0], CMD_SPI_RD_WR);
        assert_eq!(pkt[1], 2);
        assert_eq!(pkt[2], 0);
        assert_eq!(&pkt[3..], &data);
    }

    #[test]
    fn test_speed_conversion() {
        assert_eq!(SpiSpeed::from_u8(0), SpiSpeed::Speed60M);
        assert_eq!(SpiSpeed::from_u8(1), SpiSpeed::Speed30M);
        assert_eq!(SpiSpeed::from_u8(2), SpiSpeed::Speed15M);
        assert_eq!(SpiSpeed::from_u8(3), SpiSpeed::Speed7_5M);
        assert_eq!(SpiSpeed::from_u8(4), SpiSpeed::Speed3_75M);
        assert_eq!(SpiSpeed::from_u8(5), SpiSpeed::Speed1_875M);
        assert_eq!(SpiSpeed::from_u8(6), SpiSpeed::Speed937K);
        assert_eq!(SpiSpeed::from_u8(7), SpiSpeed::Speed468K);
    }
}
