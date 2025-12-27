//! Error types for nander-rs
//!
//! This module defines all error types used throughout the library.

use thiserror::Error;

/// Result type alias using our Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for nander-rs
#[derive(Error, Debug)]
pub enum Error {
    /// USB communication error
    #[error("USB error: {0}")]
    Usb(#[from] nusb::Error),

    /// Programmer not found
    #[error("Programmer not found. Is the CH341A connected?")]
    ProgrammerNotFound,

    /// Flash chip not detected
    #[error("Flash chip not detected. Check connections and power.")]
    FlashNotDetected,

    /// Unsupported flash chip
    #[error("Unsupported flash chip: JEDEC ID = {0:02X} {1:02X} {2:02X}")]
    UnsupportedChip(u8, u8, u8),

    /// Verification failed
    #[error(
        "Verification failed at address 0x{address:08X}: expected {expected:02X}, got {actual:02X}"
    )]
    VerificationFailed {
        address: u32,
        expected: u8,
        actual: u8,
    },

    /// Erase failed
    #[error("Erase failed at block {block}")]
    EraseFailed { block: u32 },

    /// Write failed
    #[error("Write failed at address 0x{address:08X}")]
    WriteFailed { address: u32 },

    /// Read failed
    #[error("Read failed at address 0x{address:08X}")]
    ReadFailed { address: u32 },

    /// ECC error (uncorrectable)
    #[error("Uncorrectable ECC error at address 0x{address:08X}")]
    EccError { address: u32 },

    /// Bad block detected
    #[error("Bad block detected at block {block}")]
    BadBlock { block: u32 },

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(std::io::Error),

    /// Transfer error (raw USB)
    #[error("USB transfer error: {0}")]
    Transfer(#[from] nusb::transfer::TransferError),
}
