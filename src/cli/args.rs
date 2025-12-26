//! CLI argument definitions using clap
//!
//! This module defines the command-line argument structure.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// nander-rs - A modern SPI NAND/NOR Flash programmer
///
/// Supports CH341A-based programmers for reading, writing, and erasing
/// SPI NAND and NOR flash chips.
#[derive(Parser, Debug)]
#[command(name = "nander")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Command to execute
    #[command(subcommand)]
    pub command: Command,
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Detect and display information about the connected flash chip
    #[command(alias = "i")]
    Info,

    /// List all supported flash chips
    #[command(alias = "L")]
    List,

    /// Read flash contents to a file
    #[command(alias = "r")]
    Read {
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Number of bytes to read (default: entire chip)
        #[arg(short, long)]
        length: Option<u32>,

        /// Start address (default: 0)
        #[arg(short, long, default_value = "0")]
        start: u32,

        /// Disable internal ECC (for NAND flash, reads raw page data including ECC bytes)
        #[arg(short = 'd', long = "no-ecc")]
        disable_ecc: bool,
    },

    /// Write a file to flash
    #[command(alias = "w")]
    Write {
        /// Input file path
        #[arg(short, long)]
        input: PathBuf,

        /// Start address (default: 0)
        #[arg(short, long, default_value = "0")]
        start: u32,

        /// Verify after writing
        #[arg(short = 'V', long, default_value = "true")]
        verify: bool,

        /// Disable internal ECC (for NAND flash, writes raw page data including ECC bytes)
        #[arg(short = 'd', long = "no-ecc")]
        disable_ecc: bool,
    },

    /// Erase flash contents
    #[command(alias = "e")]
    Erase {
        /// Number of bytes to erase (default: entire chip)
        #[arg(short, long)]
        length: Option<u32>,

        /// Start address (default: 0)
        #[arg(short, long, default_value = "0")]
        start: u32,

        /// Disable internal ECC (for NAND flash)
        #[arg(short = 'd', long = "no-ecc")]
        disable_ecc: bool,
    },

    /// Verify flash contents against a file
    #[command(alias = "v")]
    Verify {
        /// Input file path to verify against
        #[arg(short, long)]
        input: PathBuf,

        /// Start address (default: 0)
        #[arg(short, long, default_value = "0")]
        start: u32,

        /// Disable internal ECC (for NAND flash)
        #[arg(short = 'd', long = "no-ecc")]
        disable_ecc: bool,
    },
}
