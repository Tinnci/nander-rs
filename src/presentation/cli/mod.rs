//! CLI Presentation Module
//!
//! Entry point for the CLI presentation layer.

pub mod args;
pub mod handlers;

use crate::domain::bad_block::BadBlockStrategy;
use crate::error::Result;
use args::{Args, Command};
use handlers::*;

fn get_bad_block_strategy(skip: bool, include: bool) -> BadBlockStrategy {
    if include {
        BadBlockStrategy::Include
    } else if skip {
        BadBlockStrategy::Skip
    } else {
        BadBlockStrategy::Fail
    }
}

/// Execute the command specified by CLI arguments using the new architecture
pub fn execute(args: Args) -> Result<()> {
    match args.command {
        Command::Info => {
            let handler = InfoHandler::new();
            handler.handle()
        }
        Command::List => {
            let handler = ListHandler::new();
            handler.handle()
        }
        Command::Read {
            output,
            length,
            start,
            disable_ecc,
            skip_bad,
            include_bad,
        } => {
            let handler = ReadHandler::new();
            let strategy = get_bad_block_strategy(skip_bad, include_bad);
            handler.handle(output, start, length, disable_ecc, strategy)
        }
        Command::Write {
            input,
            start,
            verify,
            disable_ecc,
            skip_bad,
            include_bad,
        } => {
            let handler = WriteHandler::new();
            let strategy = get_bad_block_strategy(skip_bad, include_bad);
            handler.handle(input, start, verify, disable_ecc, strategy)
        }
        Command::Erase {
            length,
            start,
            disable_ecc: _, // Erase handler currently doesn't use ECC
            skip_bad,
            include_bad,
        } => {
            let handler = EraseHandler::new();
            let strategy = get_bad_block_strategy(skip_bad, include_bad);
            handler.handle(start, length, strategy)
        }
        Command::Verify {
            input,
            start,
            disable_ecc,
            skip_bad,
            include_bad,
        } => {
            let handler = VerifyHandler::new();
            let strategy = get_bad_block_strategy(skip_bad, include_bad);
            handler.handle(input, start, disable_ecc, strategy)
        }
    }
}
