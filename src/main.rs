//! nander-rs - A modern SPI NAND/NOR Flash programmer
//!
//! This is the main entry point for the CLI application.

use anyhow::Result;
use clap::Parser;
use log::info;

use nander_rs::presentation::cli::args::Args;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Parse command line arguments
    let args = Args::parse();

    info!("nander-rs v{}", env!("CARGO_PKG_VERSION"));

    // Execute the command using the new layered architecture
    if let Err(e) = nander_rs::presentation::cli::execute(args) {
        use colored::*;
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }

    Ok(())
}
