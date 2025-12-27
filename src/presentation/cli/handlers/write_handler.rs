//! CLI Handler - Write
//!
//! Handles the 'write' command by invoking the write flash use case.

use std::path::PathBuf;

use crate::application::use_cases::detect_chip::DetectChipUseCase;
use crate::application::use_cases::write_flash::{WriteFlashUseCase, WriteParams};
use crate::domain::FlashType;
use crate::error::{Error, Result};
use crate::infrastructure::chip_database::ChipRegistry;
use crate::infrastructure::flash_protocol::eeprom::{I2cEeprom, MicrowireEeprom, SpiEeprom};
use crate::infrastructure::flash_protocol::nand::SpiNand;
use crate::infrastructure::flash_protocol::nor::SpiNor;

pub struct WriteHandler {
    detect_use_case: DetectChipUseCase,
}

impl Default for WriteHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl WriteHandler {
    pub fn new() -> Self {
        Self {
            detect_use_case: DetectChipUseCase::new(ChipRegistry::new()),
        }
    }

    pub fn handle(&self, input: PathBuf, options: crate::domain::FlashOptions) -> Result<()> {
        let (programmer, spec) = self.detect_use_case.execute(options.speed)?;
        println!("Detected chip: {} ({})", spec.name, spec.manufacturer);

        let data = std::fs::read(input).map_err(Error::Io)?;
        let start = options.address;
        println!(
            "Writing {} bytes starting at 0x{:08X}...",
            data.len(),
            start
        );

        let params = WriteParams {
            address: start,
            data: &data,
            use_ecc: options.use_ecc,
            verify: options.verify,
            ignore_ecc_errors: options.ignore_ecc_errors,
            oob_mode: options.oob_mode,
            bad_block_strategy: options.bad_block_strategy,
            bbt: None,
        };

        let pb = super::create_progress_bar(data.len() as u64, "Writing");

        match spec.flash_type {
            FlashType::Nand => {
                let protocol = SpiNand::new(programmer, spec);
                let mut use_case = WriteFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    pb.set_position(progress.current);
                })?
            }
            FlashType::Nor => {
                let protocol = SpiNor::new(programmer, spec);
                let mut use_case = WriteFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    pb.set_position(progress.current);
                })?
            }
            FlashType::SpiEeprom => {
                let protocol = SpiEeprom::new(programmer, spec);
                let mut use_case = WriteFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    pb.set_position(progress.current);
                })?
            }
            FlashType::I2cEeprom => {
                let protocol = I2cEeprom::new(programmer, spec);
                let mut use_case = WriteFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    pb.set_position(progress.current);
                })?
            }
            FlashType::MicrowireEeprom => {
                let protocol = MicrowireEeprom::new(programmer, spec);
                let mut use_case = WriteFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    pb.set_position(progress.current);
                })?
            }
        };

        pb.finish_with_message("Write Complete");

        use colored::*;
        println!("{}", "\nWrite SUCCESSFUL!".green().bold());
        Ok(())
    }
}
