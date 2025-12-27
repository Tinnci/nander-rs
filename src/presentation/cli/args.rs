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

    /// SPI speed setting (0=~21kHz, 1=~100kHz, 2=~400kHz, 3=~750kHz, 4=~1.5MHz, 5=~3MHz, 6=~6MHz, 7=~12MHz)
    /// Higher speeds may not work with all chips. Default: 5 (~3MHz)
    #[arg(long = "speed", global = true, default_value = "5", value_parser = clap::value_parser!(u8).range(0..8))]
    pub spi_speed: u8,

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

        /// Skip bad blocks encountered during operation (NAND only)
        #[arg(short = 'k', long = "skip-bad")]
        skip_bad: bool,

        /// Include blocks even if marked bad (NAND only)
        #[arg(short = 'K', long = "include-bad")]
        include_bad: bool,

        /// Read OOB data alongside main page (NAND only)
        #[arg(short = 'o', long = "oob")]
        oob: bool,

        /// Read ONLY OOB data (NAND only)
        #[arg(short = 'O', long = "oob-only")]
        oob_only: bool,

        /// Ignore ECC errors and continue reading (NAND only)
        #[arg(short = 'I', long = "ignore-ecc")]
        ignore_ecc: bool,
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

        /// Skip bad blocks encountered during operation (NAND only)
        #[arg(short = 'k', long = "skip-bad")]
        skip_bad: bool,

        /// Include blocks even if marked bad (NAND only)
        #[arg(short = 'K', long = "include-bad")]
        include_bad: bool,

        /// Write OOB data alongside main page (NAND only)
        #[arg(short = 'o', long = "oob")]
        oob: bool,

        /// Write ONLY OOB data (NAND only)
        #[arg(short = 'O', long = "oob-only")]
        oob_only: bool,

        /// Ignore ECC errors during verify (NAND only)
        #[arg(short = 'I', long = "ignore-ecc")]
        ignore_ecc: bool,
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

        /// Skip bad blocks encountered during operation (NAND only)
        #[arg(short = 'k', long = "skip-bad")]
        skip_bad: bool,

        /// Include blocks even if marked bad (NAND only)
        #[arg(short = 'K', long = "include-bad")]
        include_bad: bool,
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

        /// Skip bad blocks encountered during operation (NAND only)
        #[arg(short = 'k', long = "skip-bad")]
        skip_bad: bool,

        /// Include blocks even if marked bad (NAND only)
        #[arg(short = 'K', long = "include-bad")]
        include_bad: bool,

        /// Verify OOB data alongside main page (NAND only)
        #[arg(short = 'o', long = "oob")]
        oob: bool,

        /// Verify ONLY OOB data (NAND only)
        #[arg(short = 'O', long = "oob-only")]
        oob_only: bool,

        /// Ignore ECC errors and continue verifying (NAND only)
        #[arg(short = 'I', long = "ignore-ecc")]
        ignore_ecc: bool,
    },

    /// Manage flash write protection (BP bits)
    Protect {
        /// Operation: status, enable, disable
        #[arg(value_name = "OPERATION", default_value = "status")]
        operation: String,
    },

    /// Directly read or write status register(s)
    Status {
        /// New status register value(s) in hex (optional, if provided it performs a write)
        #[arg(value_name = "VALUE")]
        value: Option<String>,
    },

    /// Bad Block Table management
    Bbt {
        #[command(subcommand)]
        command: BbtCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum BbtCommand {
    /// Scan chip and list all bad blocks
    Scan,
}
