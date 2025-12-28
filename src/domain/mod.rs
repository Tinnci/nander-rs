//! Domain Layer - Core Business Logic
//!
//! This layer contains the pure business logic and domain models.
//! It has NO dependencies on infrastructure or presentation layers.

pub mod bad_block;
pub mod chip;
pub mod ecc;
pub mod flash_operation;
pub mod serial_analysis;
pub mod types;

// Re-exports
pub use bad_block::{BadBlockInfo, BadBlockReason, BadBlockStrategy};
pub use chip::{BlockStatus, ChipCapabilities, ChipLayout, ChipSpec};
pub use ecc::{EccPolicy, EccStatus};
pub use flash_operation::{EraseRequest, FlashOperation, OobMode, ReadRequest, WriteRequest};
pub use types::*;
