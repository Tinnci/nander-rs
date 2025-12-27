//! CLI Handler - Erase
//!
//! Handles the 'erase' command by invoking the erase flash use case.

use crate::application::use_cases::detect_chip::DetectChipUseCase;
use crate::application::use_cases::erase_flash::{EraseFlashUseCase, EraseParams};
use crate::domain::FlashType;
use crate::error::Result;
use crate::infrastructure::chip_database::ChipRegistry;
use crate::infrastructure::flash_protocol::eeprom::{I2cEeprom, MicrowireEeprom, SpiEeprom};
use crate::infrastructure::flash_protocol::nand::SpiNand;
use crate::infrastructure::flash_protocol::nor::SpiNor;

pub struct EraseHandler {
    detect_use_case: DetectChipUseCase,
}

impl Default for EraseHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl EraseHandler {
    pub fn new() -> Self {
        Self {
            detect_use_case: DetectChipUseCase::new(ChipRegistry::new()),
        }
    }

    pub fn handle(&self, options: crate::domain::FlashOptions) -> Result<()> {
        let (programmer, spec) = self.detect_use_case.execute(options.speed)?;
        println!("Detected chip: {} ({})", spec.name, spec.manufacturer);

        let start = options.address;
        let erase_len = options.length.unwrap_or(spec.capacity.as_bytes() - start);

        let params = EraseParams {
            address: start,
            length: erase_len,
            bad_block_strategy: options.bad_block_strategy,
        };

        println!("Erasing {} bytes starting at 0x{:08X}...", erase_len, start);

        let pb = super::create_progress_bar(erase_len as u64, "Erasing");

        match spec.flash_type {
            FlashType::Nand => {
                let protocol = SpiNand::new(programmer, spec);
                let mut use_case = EraseFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    pb.set_position(progress.current);
                })?
            }
            FlashType::Nor => {
                let protocol = SpiNor::new(programmer, spec);
                let mut use_case = EraseFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    pb.set_position(progress.current);
                })?
            }
            FlashType::SpiEeprom => {
                // SPI EEPROM doesn't need explicit erase, but we fill with 0xFF
                pb.set_message("Filling with 0xFF");
                let protocol = SpiEeprom::new(programmer, spec);
                let mut use_case = EraseFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    pb.set_position(progress.current);
                })?
            }
            FlashType::I2cEeprom => {
                // I2C EEPROM doesn't need explicit erase, but we fill with 0xFF
                pb.set_message("Filling with 0xFF");
                let protocol = I2cEeprom::new(programmer, spec);
                let mut use_case = EraseFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    pb.set_position(progress.current);
                })?
            }
            FlashType::MicrowireEeprom => {
                // Microwire EEPROM doesn't need explicit erase, but we fill with 0xFF
                pb.set_message("Filling with 0xFF");
                let protocol = MicrowireEeprom::new(programmer, spec);
                let mut use_case = EraseFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    pb.set_position(progress.current);
                })?
            }
        };

        pb.finish_with_message("Erase Complete");

        println!("\nDone!");
        Ok(())
    }
}
