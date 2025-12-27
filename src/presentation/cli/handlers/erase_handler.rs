//! CLI Handler - Erase
//!
//! Handles the 'erase' command by invoking the erase flash use case.

use std::io::Write;

use crate::application::use_cases::detect_chip::DetectChipUseCase;
use crate::application::use_cases::erase_flash::{EraseFlashUseCase, EraseParams};
use crate::domain::{BadBlockStrategy, FlashType};
use crate::error::{Error, Result};
use crate::infrastructure::chip_database::ChipRegistry;
use crate::infrastructure::flash_protocol::eeprom::{I2cEeprom, SpiEeprom};
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

    pub fn handle(
        &self,
        start: u32,
        length: Option<u32>,
        strategy: BadBlockStrategy,
        speed: Option<u8>,
    ) -> Result<()> {
        let (programmer, spec) = self.detect_use_case.execute(speed)?;
        println!("Detected chip: {} ({})", spec.name, spec.manufacturer);

        let erase_len = length.unwrap_or(spec.capacity.as_bytes() - start);

        let params = EraseParams {
            address: start,
            length: erase_len,
            bad_block_strategy: strategy,
        };

        println!("Erasing {} bytes starting at 0x{:08X}...", erase_len, start);

        match spec.flash_type {
            FlashType::Nand => {
                let protocol = SpiNand::new(programmer, spec);
                let mut use_case = EraseFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    print!("\rProgress: {:.1}%", progress.percentage());
                    let _ = std::io::stdout().flush();
                })?
            }
            FlashType::Nor => {
                let protocol = SpiNor::new(programmer, spec);
                let mut use_case = EraseFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    print!("\rProgress: {:.1}%", progress.percentage());
                    let _ = std::io::stdout().flush();
                })?
            }
            FlashType::SpiEeprom => {
                // SPI EEPROM doesn't need explicit erase, but we fill with 0xFF
                println!("Note: SPI EEPROM will be filled with 0xFF (no explicit erase needed)");
                let protocol = SpiEeprom::new(programmer, spec);
                let mut use_case = EraseFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    print!("\rProgress: {:.1}%", progress.percentage());
                    let _ = std::io::stdout().flush();
                })?
            }
            FlashType::I2cEeprom => {
                // I2C EEPROM doesn't need explicit erase, but we fill with 0xFF
                println!("Note: I2C EEPROM will be filled with 0xFF (no explicit erase needed)");
                let protocol = I2cEeprom::new(programmer, spec);
                let mut use_case = EraseFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    print!("\rProgress: {:.1}%", progress.percentage());
                    let _ = std::io::stdout().flush();
                })?
            }
            FlashType::MicrowireEeprom => {
                return Err(Error::NotSupported(
                    "Microwire EEPROM support is not yet implemented".to_string(),
                ));
            }
        };

        println!("\nDone!");
        Ok(())
    }
}
