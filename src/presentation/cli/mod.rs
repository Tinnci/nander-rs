//! CLI Presentation Module
//!
//! Entry point for the CLI presentation layer.

use crate::cli::args::{self, Args};
use crate::error::Result;
use handlers::*;

/// Execute the command specified by CLI arguments using the new architecture
pub fn execute(args: Args) -> Result<()> {
    match args.command {
        args::Command::Info => {
            let handler = InfoHandler::new();
            handler.handle()
        }
        args::Command::List => {
            let handler = ListHandler::new();
            handler.handle()
        }
        args::Command::Read {
            output,
            length,
            start,
            disable_ecc,
        } => {
            let handler = ReadHandler::new();
            handler.handle(output, start, length, disable_ecc)
        }
        args::Command::Write {
            input,
            start,
            verify,
            disable_ecc,
        } => {
            let handler = WriteHandler::new();
            handler.handle(input, start, verify, disable_ecc)
        }
        args::Command::Erase {
            length,
            start,
            disable_ecc: _, // Erase handler currently doesn't use ECC
        } => {
            let handler = EraseHandler::new();
            handler.handle(start, length)
        }
        args::Command::Verify {
            input,
            start,
            disable_ecc,
        } => {
            let handler = VerifyHandler::new();
            handler.handle(input, start, disable_ecc)
        }
    }
}
