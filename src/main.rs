//! nander-rs - A modern SPI NAND/NOR Flash programmer
//!
//! This is the main entry point for the CLI application.

mod cli;
mod database;
mod error;
mod flash;
mod hardware;

use anyhow::Result;
use clap::Parser;
use log::info;

use cli::Args;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Parse command line arguments
    let args = Args::parse();

    info!("nander-rs v{}", env!("CARGO_PKG_VERSION"));

    // Execute the command
    cli::execute(args)?;

    Ok(())
}
