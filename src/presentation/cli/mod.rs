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
            handler.handle(Some(args.spi_speed), Some(&args.driver))
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
            retries,
            bbt_file,
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
                retry_count: retries,
                bbt_file,
                driver: Some(args.driver.clone()),
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
            retries,
            bbt_file,
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
                retry_count: retries,
                bbt_file,
                driver: Some(args.driver.clone()),
            };
            handler.handle(input, options)
        }
        Command::Erase {
            length,
            start,
            disable_ecc: _,
            skip_bad,
            include_bad,
            bbt_file,
        } => {
            let handler = EraseHandler::new();
            let options = FlashOptions {
                address: start,
                length,
                bad_block_strategy: get_bad_block_strategy(skip_bad, include_bad),
                speed: Some(args.spi_speed),
                bbt_file,
                driver: Some(args.driver.clone()),
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
            retries,
            bbt_file,
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
                retry_count: retries,
                bbt_file,
                driver: Some(args.driver.clone()),
            };
            handler.handle(input, options)
        }
        Command::Protect { operation } => {
            let handler = ProtectHandler::new();
            handler.handle_protect(&operation, Some(args.spi_speed), Some(&args.driver))
        }
        Command::Status { value } => {
            let handler = ProtectHandler::new();
            handler.handle_status(value, Some(args.spi_speed), Some(&args.driver))
        }
        Command::Bbt { command } => {
            let handler = BbtHandler::new();
            match command {
                args::BbtCommand::Scan { output } => {
                    handler.handle_scan(Some(args.spi_speed), output, Some(&args.driver))
                }
                args::BbtCommand::Load { input } => handler.handle_load(input),
            }
        }
        Command::Diagnostic { interactive } => {
            use crate::application::DiagnosticTool;
            use crate::infrastructure::programmer;

            // Discover programmer (doesn't need chip detection)
            let mut prog = programmer::discover(Some(&args.driver))?;
            if let Some(speed) = Some(args.spi_speed) {
                prog.set_speed(speed)?;
            }

            if interactive {
                DiagnosticTool::interactive_spi_test(prog.as_mut())?;
            } else {
                DiagnosticTool::run_diagnostics(prog.as_mut())?;
            }
            Ok(())
        }
        Command::Batch {
            script,
            template,
            firmware,
            save_to,
        } => {
            use crate::application::batch::{templates, BatchScript};
            use crate::infrastructure::chip_database::ChipRegistry;
            use crate::infrastructure::programmer;

            // Load or generate the batch script
            let batch_script = if let Some(script_path) = script {
                // Load from file (auto-detect JSON/TOML from extension)
                let ext = script_path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                match ext {
                    "json" => BatchScript::from_json_file(&script_path)?,
                    "toml" => BatchScript::from_toml_file(&script_path)?,
                    _ => {
                        return Err(crate::error::Error::Other(
                            "Script file must have .json or .toml extension".to_string(),
                        ))
                    }
                }
            } else if let Some(template_name) = template {
                // Use built-in template
                let firmware_path = firmware.ok_or_else(|| {
                    crate::error::Error::Other(
                        "Firmware file is required when using templates".to_string(),
                    )
                })?;

                match template_name.as_str() {
                    "flash-update" => templates::flash_update(firmware_path),
                    "production" => templates::production_program(firmware_path),
                    _ => {
                        return Err(crate::error::Error::Other(format!(
                            "Unknown template: '{}'. Available: 'flash-update', 'production'",
                            template_name
                        )))
                    }
                }
            } else {
                return Err(crate::error::Error::Other(
                    "Either --script or --template must be specified".to_string(),
                ));
            };

            // If --save-to is specified, save the script and exit
            if let Some(save_path) = save_to {
                let ext = save_path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("json");
                match ext {
                    "json" => batch_script.to_json_file(&save_path)?,
                    "toml" => batch_script.to_toml_file(&save_path)?,
                    _ => batch_script.to_json_file(&save_path)?,
                }
                println!("✓ Batch script saved to: {:?}", save_path);
                return Ok(());
            }

            // Execute the batch script
            let mut prog = programmer::discover(Some(&args.driver))?;
            if let Some(speed) = Some(args.spi_speed) {
                prog.set_speed(speed)?;
            }

            let registry = ChipRegistry::new();
            batch_script.execute(prog.as_mut(), &registry)?;

            Ok(())
        }
        Command::Gui => {
            crate::presentation::gui::run().map_err(|e| crate::error::Error::Other(e.to_string()))
        }
    }
}
