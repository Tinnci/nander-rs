//! Domain Model - Bad Block Management
//!
//! This module defines bad block handling strategies and information,
//! independent of implementation details.

use super::types::Address;

/// Information about a bad block
#[derive(Debug, Clone)]
pub struct BadBlockInfo {
    /// Block address
    pub address: Address,
    /// Reason the block is marked bad
    pub reason: BadBlockReason,
}

/// Reason a block is marked as bad
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadBlockReason {
    /// Factory marked bad block
    Factory,
    /// Bad block detected during runtime (failed erase/program)
    Runtime,
    /// Bad block detected during read (uncorrectable ECC errors)
    EccFailure,
    /// Manually marked by user
    Manual,
}

/// Strategy for handling bad blocks during operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadBlockStrategy {
    /// Stop operation when a bad block is encountered
    #[default]
    Fail,
    /// Skip bad blocks and continue with the next good block
    Skip,
    /// Include bad blocks in the operation (for raw dumps)
    Include,
}

impl BadBlockStrategy {
    /// Returns true if the strategy allows continuing past bad blocks
    pub fn should_continue(&self) -> bool {
        matches!(self, Self::Skip | Self::Include)
    }

    /// Returns true if bad blocks should be included in the data
    pub fn should_include_bad(&self) -> bool {
        matches!(self, Self::Include)
    }
}
