use crate::domain::{ChipSpec, Progress};
use std::path::PathBuf;

/// Messages sent from the UI/Main thread to the Background Worker
pub enum GuiMessage {
    /// Request to connect to a programmer
    Connect,
    /// Request to detect chip
    DetectChip,
    /// Request to read flash
    ReadFlash {
        path: PathBuf,
        start: u32,
        length: Option<u32>,
    },
    /// Request to write flash
    WriteFlash {
        path: PathBuf,
        start: u32,
        verify: bool,
    },
    /// Request to erase flash
    EraseFlash { start: u32, length: Option<u32> },
    /// Request to cancel current operation
    Cancel,
}

/// Messages sent from the Background Worker to the UI
pub enum WorkerMessage {
    /// Connection successful
    Connected(String), // Programmer name
    /// Connection failed
    ConnectionFailed(String), // Error message
    /// Chip detected
    ChipDetected(ChipSpec),
    /// Chip detection failed
    ChipDetectionFailed(String),
    /// Progress update
    Progress(Progress),
    /// Operation completed
    OperationComplete,
    /// Data read from flash (for preview)
    DataRead(Vec<u8>),
    /// Operation failed
    OperationFailed(String),
    /// Log message
    Log(String),
    /// List of detected devices (for diagnostic display)
    DeviceList(Vec<String>),
}
