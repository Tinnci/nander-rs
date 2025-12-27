//! CLI Handlers Module
//!
//! Contains individual command handlers for the CLI.

pub mod bbt_handler;
pub mod erase_handler;
pub mod info_handler;
pub mod list_handler;
pub mod protect_handler;
pub mod read_handler;
pub mod verify_handler;
pub mod write_handler;

pub use bbt_handler::BbtHandler;
pub use erase_handler::EraseHandler;
pub use info_handler::InfoHandler;
pub use list_handler::ListHandler;
pub use protect_handler::ProtectHandler;
pub use read_handler::ReadHandler;
pub use verify_handler::VerifyHandler;
pub use write_handler::WriteHandler;

use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::time::Duration;

use crate::domain::bad_block::BadBlockTable;
use crate::error::{Error, Result};

/// Create a standardized, stylish progress bar for flash operations
pub fn create_progress_bar(total_size: u64, message: &'static str) -> ProgressBar {
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta}) {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(message);
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Load a Bad Block Table from a JSON file
pub fn load_bbt(path: &PathBuf) -> Result<BadBlockTable> {
    let file = std::fs::File::open(path).map_err(Error::Io)?;
    let bbt: BadBlockTable = serde_json::from_reader(file)
        .map_err(|e| Error::Other(format!("Failed to parse BBT file: {}", e)))?;
    Ok(bbt)
}
