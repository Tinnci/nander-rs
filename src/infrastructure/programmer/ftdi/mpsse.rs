//! MPSSE Command Builder
//!
//! Constants and helper functions for constructing MPSSE byte commands.
//! Reference: FTDI AN_108

#![allow(dead_code)]

// --- Command Opcodes ---

// Data Transfer Commands (MSB First for SPI)
pub const CMD_MSB_DATA_OUT_BYTES_POS: u8 = 0x10;
pub const CMD_MSB_DATA_OUT_BYTES_NEG: u8 = 0x11;
pub const CMD_MSB_DATA_IN_BYTES_POS: u8 = 0x20; // Sample on rising edge
pub const CMD_MSB_DATA_IN_BYTES_NEG: u8 = 0x24; // Sample on falling edge
pub const CMD_MSB_DATA_IN_OUT_BYTES_POS: u8 = 0x31; // Out -ve, In +ve (Mode 0)
pub const CMD_MSB_DATA_IN_OUT_BYTES_NEG: u8 = 0x34; // Out +ve, In -ve

// Data Transfer Commands (Bits)
pub const CMD_MSB_DATA_OUT_BITS_POS: u8 = 0x12;
pub const CMD_MSB_DATA_OUT_BITS_NEG: u8 = 0x13;
pub const CMD_MSB_DATA_IN_BITS_POS: u8 = 0x22;
pub const CMD_MSB_DATA_IN_BITS_NEG: u8 = 0x26;
pub const CMD_MSB_DATA_IN_OUT_BITS_POS: u8 = 0x33;
pub const CMD_MSB_DATA_IN_OUT_BITS_NEG: u8 = 0x36;

// GPIO / Setup Commands
pub const CMD_SET_BITS_LOW: u8 = 0x80; // ADBUS (TCK, TDI, TDO, TMS, GPIOL0-3)
pub const CMD_SET_BITS_HIGH: u8 = 0x82; // ACBUS (GPIOH0-7)
pub const CMD_READ_BITS_LOW: u8 = 0x81;
pub const CMD_READ_BITS_HIGH: u8 = 0x83;
pub const CMD_LOOPBACK_ON: u8 = 0x84;
pub const CMD_LOOPBACK_OFF: u8 = 0x85;
pub const CMD_SET_TCK_DIVISOR: u8 = 0x86;
pub const CMD_SEND_IMMEDIATE: u8 = 0x87; // Flush buffer to PC
pub const CMD_WAIT_ON_IO_HIGH: u8 = 0x88;
pub const CMD_WAIT_ON_IO_LOW: u8 = 0x89;

// --- Helper Functions ---

/// Build command to set TCK divisor
/// TCK = 60MHz / ((1 + Divisor) * 2)
/// Divisor = (60MHz / 2*TCK) - 1
pub fn build_set_divisor_cmd(divisor: u16) -> [u8; 3] {
    [
        CMD_SET_TCK_DIVISOR,
        (divisor & 0xFF) as u8,
        ((divisor >> 8) & 0xFF) as u8,
    ]
}

/// Build command to set ADBUS (Low Byte) GPIO Direction and Value
/// ADBUS0: TCK (Out)
/// ADBUS1: TDI (Out)
/// ADBUS2: TDO (In)
/// ADBUS3: TMS (Out - CS usually)
/// ADBUS4-7: GPIO
pub fn build_set_low_gpio_cmd(value: u8, direction: u8) -> [u8; 3] {
    [CMD_SET_BITS_LOW, value, direction]
}

/// Build command to set ACBUS (High Byte) GPIO Direction and Value
pub fn build_set_high_gpio_cmd(value: u8, direction: u8) -> [u8; 3] {
    [CMD_SET_BITS_HIGH, value, direction]
}

/// Build SPI write bytes command (Mode 0: MSB First, Write on Falling Edge)
/// Max length 65536 bytes per command block
pub fn build_write_bytes_cmd(data: &[u8]) -> Vec<u8> {
    let len = data.len();
    if len == 0 {
        return Vec::new();
    }
    // Length is len - 1
    let len_val = (len - 1) as u16;
    let mut cmd = Vec::with_capacity(3 + len);
    cmd.push(CMD_MSB_DATA_OUT_BYTES_NEG);
    cmd.push((len_val & 0xFF) as u8);
    cmd.push(((len_val >> 8) & 0xFF) as u8);
    cmd.extend_from_slice(data);
    cmd
}

/// Build SPI read bytes command (Mode 0: MSB First, Read on Rising Edge)
pub fn build_read_bytes_cmd(len: usize) -> Vec<u8> {
    if len == 0 {
        return Vec::new();
    }
    let len_val = (len - 1) as u16;
    vec![
        CMD_MSB_DATA_IN_BYTES_POS,
        (len_val & 0xFF) as u8,
        ((len_val >> 8) & 0xFF) as u8,
    ]
}

/// Build SPI read/write bytes command (Mode 0)
pub fn build_rw_bytes_cmd(data: &[u8]) -> Vec<u8> {
    let len = data.len();
    if len == 0 {
        return Vec::new();
    }
    let len_val = (len - 1) as u16;
    let mut cmd = Vec::with_capacity(3 + len);
    cmd.push(CMD_MSB_DATA_IN_OUT_BYTES_POS); // Out -ve, In +ve
    cmd.push((len_val & 0xFF) as u8);
    cmd.push(((len_val >> 8) & 0xFF) as u8);
    cmd.extend_from_slice(data);
    cmd
}

/// Build Loopback command
pub fn build_loopback_cmd(enable: bool) -> u8 {
    if enable {
        CMD_LOOPBACK_ON
    } else {
        CMD_LOOPBACK_OFF
    }
}
