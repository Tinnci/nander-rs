//! CLI Presentation Module
//!
//! Entry point for the CLI presentation layer.

pub mod args;
pub mod handlers;

use crate::domain::bad_block::BadBlockStrategy;
use crate::domain::{FlashOptions, OobMode};
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

fn get_oob_mode(oob: bool, oob_only: bool) -> OobMode {
    if oob_only {
        OobMode::Only
    } else if oob {
        OobMode::Included
    } else {
        OobMode::None
    }
}

/// Execute the command specified by CLI arguments using the new architecture
pub fn execute(args: Args) -> Result<()> {
    match args.command {
        Command::Info => {
            let handler = InfoHandler::new();
            handler.handle(Some(args.spi_speed))
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
            oob,
            oob_only,
            ignore_ecc,
        } => {
            let handler = ReadHandler::new();
            let options = FlashOptions {
                address: start,
                length,
                use_ecc: !disable_ecc,
                ignore_ecc_errors: ignore_ecc,
                bad_block_strategy: get_bad_block_strategy(skip_bad, include_bad),
                oob_mode: get_oob_mode(oob, oob_only),
                speed: Some(args.spi_speed),
                verify: false,
            };
            handler.handle(output, options)
        }
        Command::Write {
            input,
            start,
            verify,
            disable_ecc,
            skip_bad,
            include_bad,
            oob,
            oob_only,
            ignore_ecc,
        } => {
            let handler = WriteHandler::new();
            let options = FlashOptions {
                address: start,
                length: None, // Write uses input file length
                use_ecc: !disable_ecc,
                ignore_ecc_errors: ignore_ecc,
                bad_block_strategy: get_bad_block_strategy(skip_bad, include_bad),
                oob_mode: get_oob_mode(oob, oob_only),
                speed: Some(args.spi_speed),
                verify,
            };
            handler.handle(input, options)
        }
        Command::Erase {
            length,
            start,
            disable_ecc: _,
            skip_bad,
            include_bad,
        } => {
            let handler = EraseHandler::new();
            let options = FlashOptions {
                address: start,
                length,
                bad_block_strategy: get_bad_block_strategy(skip_bad, include_bad),
                speed: Some(args.spi_speed),
                ..Default::default()
            };
            handler.handle(options)
        }
        Command::Verify {
            input,
            start,
            disable_ecc,
            skip_bad,
            include_bad,
            oob,
            oob_only,
            ignore_ecc,
        } => {
            let handler = VerifyHandler::new();
            let options = FlashOptions {
                address: start,
                length: None,
                use_ecc: !disable_ecc,
                ignore_ecc_errors: ignore_ecc,
                bad_block_strategy: get_bad_block_strategy(skip_bad, include_bad),
                oob_mode: get_oob_mode(oob, oob_only),
                speed: Some(args.spi_speed),
                verify: false,
            };
            handler.handle(input, options)
        }
        Command::Protect { operation } => {
            let handler = ProtectHandler::new();
            handler.handle_protect(&operation, Some(args.spi_speed))
        }
        Command::Status { value } => {
            let handler = ProtectHandler::new();
            handler.handle_status(value, Some(args.spi_speed))
        }
        Command::Bbt { command } => {
            let handler = BbtHandler::new();
            match command {
                args::BbtCommand::Scan => handler.handle_scan(Some(args.spi_speed)),
            }
        }
    }
}
