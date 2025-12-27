//! CLI Handler - Bad Block Table
//!
//! Handles 'bbt scan' and other BBT commands.

use crate::application::use_cases::detect_chip::DetectChipUseCase;
use crate::domain::{bad_block::BlockStatus, FlashOperation, FlashType};
use crate::error::{Error, Result};
use crate::infrastructure::chip_database::ChipRegistry;
use crate::infrastructure::flash_protocol::nand::SpiNand;
use colored::*;

pub struct BbtHandler {
    detect_use_case: DetectChipUseCase,
}

impl Default for BbtHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl BbtHandler {
    pub fn new() -> Self {
        Self {
            detect_use_case: DetectChipUseCase::new(ChipRegistry::new()),
        }
    }

    pub fn handle_scan(&self, speed: Option<u8>) -> Result<()> {
        println!("Detecting flash chip...");
        let (programmer, spec) = self.detect_use_case.execute(speed)?;

        // BBT is only relevant for NAND
        if spec.flash_type != FlashType::Nand {
            return Err(Error::NotSupported(
                "BBT scan is only available for NAND flash".to_string(),
            ));
        }

        println!(
            "Detected: {} ({})",
            spec.name.green().bold(),
            spec.manufacturer.green()
        );
        println!("Scanning for bad blocks... (This may take a while)");

        // Note: We instantiate SpiNand directly but use it via FlashOperation trait methods if needed
        let mut protocol = SpiNand::new(programmer, spec.clone());
        let total_blocks = (spec.capacity.as_bytes() / spec.layout.block_size) as u64;

        let pb = super::create_progress_bar(total_blocks, "Scanning Blocks");

        let bbt = protocol.scan_bbt(&|progress| {
            pb.set_position(progress.current);
        })?;

        pb.finish_with_message("Scan Complete");
        println!("\n{}", "Scan Results:".cyan().bold());

        let bad_count = bbt.bad_block_count();
        if bad_count == 0 {
            println!("No bad blocks found! {}", "Excellent!".green());
        } else {
            println!("Found {} bad blocks:", bad_count.to_string().red().bold());
            println!("--------------------------------");
            println!("{:<10} {:<15}", "Block", "Status");
            println!("--------------------------------");

            let total_blocks_usize = total_blocks as usize;
            for block in 0..total_blocks_usize {
                let status = bbt.get_status(block);
                if status == BlockStatus::BadFactory {
                    println!("{:<10} {}", block, "Factory Bad".red());
                } else if status == BlockStatus::BadRuntime {
                    println!("{:<10} {}", block, "Runtime Bad".red());
                }
            }
        }

        Ok(())
    }
}
