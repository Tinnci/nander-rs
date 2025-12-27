# Bad Block Table (BBT) Design

## 1. Overview
The goal is to implement an efficient, in-memory Bad Block Table (BBT) for SPI NAND flash. This will replace the current method of checking OOB markers on-the-fly for every operation, which is slow and inefficient.

## 2. Architecture

### 2.1. Domain Layer: `BadBlockTable`
A struct representing the state of all blocks in the chip.

```rust
pub struct BadBlockTable {
    /// Total number of blocks in the chip
    total_blocks: u32,
    /// Bitmap or Byte-map status of each block.
    /// 0: Unknown/Unscanned
    /// 1: Good
    /// 2: Bad (Factory)
    /// 3: Bad (Runtime/Wear)
    status: Vec<BlockStatus>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum BlockStatus {
    Unknown,
    Good,
    BadFactory,
    BadRuntime,
}
```

### 2.2. Operations
- **Scan**: Iterate through all blocks, reading their OOB markers, and populating `status`.
- **Query**: `is_bad(block_addr) -> bool`
- **Mark**: `mark_bad(block_addr, status)`
- **Export/Import**: Serialize to/from file (for caching).

### 2.3. Strategy Update
The scanning logic currently in `is_bad_block` inside `SpiNand` should be promoted to a `scan_bbt` method.

## 3. Implementation Plan

### Phase 1: In-Memory BBT
1. Define `BadBlockTable` struct in `src/domain/bad_block.rs`.
2. Add `scan_bbt()` method to `FlashOperation` trait (default impl returns empty/error).
3. Implement `scan_bbt()` in `SpiNand`.
   - Iterates all blocks.
   - Using optimized bulk read (if possible) or page read.
   - Updates the table.
4. Modify `read`, `write`, `erase` in `SpiNand` to accept an optional `&BadBlockTable`.
   - If provided, use it.
   - If not, fall back to legacy on-the-fly checking (or force a scan first).

### Phase 2: CLI Integration
1. Add `bbt` subcommands:
   - `nander bbt scan`: Scans chip and reports bad blocks.
   - `nander bbt dump`: Dumps the table to a file.
2. Update `read`/`write` commands to optionally use a cached BBT file or scan-before-op.

## 4. Performance Considerations
Scanning a 128MB NAND (1024 blocks) involves 2048 page reads (checking 1st and 2nd page). At ~300KB/s (current unoptimized speed?) this could take a few seconds. With optimized bulk transfer, it should be faster.

## 5. Factory Marker Location
Standard assumes 1st byte of OOB in 1st or 2nd page.
Some manufacturers (Samsung, etc.) might use last page.
The scanner should start with the standard (1st/2nd page) but be extensible.
