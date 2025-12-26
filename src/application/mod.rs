//! Application Layer
//!
//! Orchestrates the business logic by coordinating domain and infrastructure.

pub mod services;
pub mod use_cases;

// Re-export commonly used types
pub use use_cases::{
    DetectChipUseCase, EraseFlashUseCase, EraseParams, ReadFlashUseCase, ReadParams,
    WriteFlashUseCase, WriteParams,
};
