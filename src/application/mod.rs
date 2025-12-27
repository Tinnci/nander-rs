//! Application Layer
//!
//! Orchestrates the business logic by coordinating domain and infrastructure.

pub mod batch;
pub mod diagnostics;
pub mod services;
pub mod use_cases;

// Re-export commonly used types
pub use batch::{BatchOperation, BatchScript};
pub use diagnostics::DiagnosticTool;
pub use use_cases::{
    DetectChipUseCase, EraseFlashUseCase, EraseParams, ReadFlashUseCase, ReadParams,
    WriteFlashUseCase, WriteParams,
};
