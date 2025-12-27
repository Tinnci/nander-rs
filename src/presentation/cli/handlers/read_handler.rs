//! CLI Handler - Read
//!
//! Handles the 'read' command by invoking the read flash use case.

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::application::use_cases::detect_chip::DetectChipUseCase;
use crate::application::use_cases::read_flash::{ReadFlashUseCase, ReadParams};
use crate::domain::FlashType;
use crate::error::{Error, Result};
use crate::infrastructure::chip_database::ChipRegistry;
use crate::infrastructure::flash_protocol::eeprom::{I2cEeprom, MicrowireEeprom, SpiEeprom};
use crate::infrastructure::flash_protocol::nand::SpiNand;
use crate::infrastructure::flash_protocol::nor::SpiNor;

pub struct ReadHandler {
    detect_use_case: DetectChipUseCase,
}

impl Default for ReadHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadHandler {
    pub fn new() -> Self {
        Self {
            detect_use_case: DetectChipUseCase::new(ChipRegistry::new()),
        }
    }

    pub fn handle(&self, output: PathBuf, options: crate::domain::FlashOptions) -> Result<()> {
        let (programmer, spec) = self.detect_use_case.execute(options.speed)?;
        println!("Detected chip: {} ({})", spec.name, spec.manufacturer);

        let start = options.address;
        let read_len = options.length.unwrap_or(spec.capacity.as_bytes() - start);

        // Prepare parameters
        let params = ReadParams {
            address: start,
            length: read_len,
            use_ecc: options.use_ecc,
            ignore_ecc_errors: options.ignore_ecc_errors,
            oob_mode: options.oob_mode,
            bad_block_strategy: options.bad_block_strategy,
            bbt: None,
        };

        println!("Reading {} bytes starting at 0x{:08X}...", read_len, start);

        let pb = super::create_progress_bar(read_len as u64, "Reading");

        let data = match spec.flash_type {
            FlashType::Nand => {
                let protocol = SpiNand::new(programmer, spec);
                let mut use_case = ReadFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    pb.set_position(progress.current);
                })?
            }
            FlashType::Nor => {
                let protocol = SpiNor::new(programmer, spec);
                let mut use_case = ReadFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    pb.set_position(progress.current);
                })?
            }
            FlashType::SpiEeprom => {
                let protocol = SpiEeprom::new(programmer, spec);
                let mut use_case = ReadFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    pb.set_position(progress.current);
                })?
            }
            FlashType::I2cEeprom => {
                let protocol = I2cEeprom::new(programmer, spec);
                let mut use_case = ReadFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    pb.set_position(progress.current);
                })?
            }
            FlashType::MicrowireEeprom => {
                let protocol = MicrowireEeprom::new(programmer, spec);
                let mut use_case = ReadFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    pb.set_position(progress.current);
                })?
            }
        };

        pb.finish_with_message("Read Complete");

        println!("\nWriting to file: {:?}", output);
        let mut file = File::create(output).map_err(Error::Io)?;
        file.write_all(&data).map_err(Error::Io)?;

        use colored::*;
        println!("{}", "Read SUCCESSFUL!".green().bold());
        Ok(())
    }
}
