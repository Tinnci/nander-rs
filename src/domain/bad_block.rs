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
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

/// Status of a flash block
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum BlockStatus {
    /// Status unknown (not scanned)
    #[default]
    Unknown,
    /// Verified Good block
    Good,
    /// Factory marked bad block
    BadFactory,
    /// Bad block detected during runtime
    BadRuntime,
}

/// In-memory Bad Block Table
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BadBlockTable {
    status: Vec<BlockStatus>,
}

impl BadBlockTable {
    pub fn new(total_blocks: usize) -> Self {
        Self {
            status: vec![BlockStatus::Unknown; total_blocks],
        }
    }

    pub fn len(&self) -> usize {
        self.status.len()
    }

    pub fn is_empty(&self) -> bool {
        self.status.is_empty()
    }

    pub fn set_status(&mut self, block: usize, status: BlockStatus) {
        if block < self.status.len() {
            self.status[block] = status;
        }
    }

    pub fn get_status(&self, block: usize) -> BlockStatus {
        if block < self.status.len() {
            self.status[block]
        } else {
            BlockStatus::Unknown
        }
    }

    pub fn is_bad(&self, block: usize) -> bool {
        matches!(
            self.get_status(block),
            BlockStatus::BadFactory | BlockStatus::BadRuntime
        )
    }

    pub fn bad_block_count(&self) -> usize {
        self.status
            .iter()
            .filter(|&&s| s == BlockStatus::BadFactory || s == BlockStatus::BadRuntime)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bad_block_strategy_logic() {
        // Test Fail strategy
        let fail = BadBlockStrategy::Fail;
        assert!(!fail.should_continue());
        assert!(!fail.should_include_bad());

        // Test Skip strategy
        let skip = BadBlockStrategy::Skip;
        assert!(skip.should_continue());
        assert!(!skip.should_include_bad());

        // Test Include strategy
        let include = BadBlockStrategy::Include;
        assert!(include.should_continue());
        assert!(include.should_include_bad());
    }

    #[test]
    fn test_bad_block_table_operations() {
        let mut bbt = BadBlockTable::new(10);
        assert_eq!(bbt.len(), 10);
        assert!(!bbt.is_empty()); // actually checks if status vector is empty, not if it contains bad blocks. status vector size 10 is not empty.

        // Check initial state
        assert_eq!(bbt.get_status(0), BlockStatus::Unknown);
        assert!(!bbt.is_bad(0));

        // Mark bad blocks
        bbt.set_status(1, BlockStatus::BadFactory);
        bbt.set_status(5, BlockStatus::BadRuntime);
        bbt.set_status(8, BlockStatus::Good);

        // Verify status
        assert_eq!(bbt.get_status(1), BlockStatus::BadFactory);
        assert!(bbt.is_bad(1));

        assert_eq!(bbt.get_status(5), BlockStatus::BadRuntime);
        assert!(bbt.is_bad(5));

        assert_eq!(bbt.get_status(8), BlockStatus::Good);
        assert!(!bbt.is_bad(8));

        // Out of bounds check
        assert_eq!(bbt.get_status(100), BlockStatus::Unknown);

        // Count check
        assert_eq!(bbt.bad_block_count(), 2);
    }
}
