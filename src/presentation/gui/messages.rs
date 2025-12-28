use crate::domain::{ChipSpec, Progress};
use crate::infrastructure::programmer::traits::SerialConfig;
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
    /// Request to set SPI speed
    SetSpeed(u8),
    /// Request to select CS line
    SetCsIndex(u8),
    /// Request to cancel current operation
    Cancel,

    // =========================================================================
    // Serial/Console Messages
    // =========================================================================
    /// Connect to a serial port
    SerialConnect,
    /// Disconnect from serial port
    SerialDisconnect,
    /// Configure serial port parameters
    SerialConfigure(SerialConfig),
    /// Send data to serial port
    SerialSend(Vec<u8>),
    /// Set DTR line
    SerialSetDtr(bool),
    /// Set RTS line
    SerialSetRts(bool),
    /// Auto-detect baud rate
    SerialAutoDetectBaud,
}

/// Messages sent from the Background Worker to the UI
pub enum WorkerMessage {
    /// Connection successful
    Connected(String), // Programmer name
    /// Connection failed
    ConnectionFailed(String), // Error message
    /// Programmer disconnected
    Disconnected,
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

    // =========================================================================
    // Serial/Console Messages
    // =========================================================================
    /// Serial port connected
    SerialConnected(String), // Port name
    /// Serial port disconnected
    SerialDisconnected,
    /// Serial connection failed
    SerialConnectionFailed(String),
    /// Received data from serial port
    SerialDataReceived(Vec<u8>),
    /// Serial send complete
    SerialSendComplete(usize), // bytes sent
    /// Auto-detect baud rate progress
    SerialAutoDetectProgress(f32),
    /// Auto-detect baud rate results (baud_rate, confidence, preview, protocol)
    SerialBaudDetectionResults(Vec<(u32, f32, String, String)>),
}
