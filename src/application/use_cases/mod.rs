//! Application Use Cases
//!
//! Business logic orchestration for flash operations.

pub mod detect_chip;
pub mod erase_flash;
pub mod read_flash;
pub mod verify_flash;
pub mod write_flash;

// Re-export use cases
pub use detect_chip::DetectChipUseCase;
pub use erase_flash::{EraseFlashUseCase, EraseParams};
pub use read_flash::{ReadFlashUseCase, ReadParams};
pub use write_flash::{WriteFlashUseCase, WriteParams};
