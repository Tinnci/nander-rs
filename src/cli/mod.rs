//! Command-line interface definitions and handlers
//!
//! This module defines the CLI structure using clap and implements
//! the command handlers for each operation.

mod args;
mod commands;

pub use args::Args;

use anyhow::Result;

/// Execute the command specified by CLI arguments
pub fn execute(args: Args) -> Result<()> {
    match args.command {
        args::Command::Info => commands::info(),
        args::Command::List => commands::list(),
        args::Command::Read {
            output,
            length,
            start,
            disable_ecc,
        } => commands::read(&output, length, start, disable_ecc),
        args::Command::Write {
            input,
            start,
            verify,
            disable_ecc,
        } => commands::write(&input, start, verify, disable_ecc),
        args::Command::Erase {
            length,
            start,
            disable_ecc,
        } => commands::erase(length, start, disable_ecc),
        args::Command::Verify {
            input,
            start,
            disable_ecc,
        } => commands::verify(&input, start, disable_ecc),
    }
}
