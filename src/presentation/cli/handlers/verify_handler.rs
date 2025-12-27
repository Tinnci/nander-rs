use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::application::use_cases::detect_chip::DetectChipUseCase;
use crate::application::use_cases::verify_flash::{VerifyFlashUseCase, VerifyParams};
use crate::domain::{BadBlockStrategy, FlashType, OobMode};
use crate::error::{Error, Result};
use crate::infrastructure::chip_database::ChipRegistry;
use crate::infrastructure::flash_protocol::eeprom::{I2cEeprom, MicrowireEeprom, SpiEeprom};
use crate::infrastructure::flash_protocol::nand::SpiNand;
use crate::infrastructure::flash_protocol::nor::SpiNor;

pub struct VerifyHandler {
    detect_use_case: DetectChipUseCase,
}

impl Default for VerifyHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl VerifyHandler {
    pub fn new() -> Self {
        Self {
            detect_use_case: DetectChipUseCase::new(ChipRegistry::new()),
        }
    }

    pub fn handle(
        &self,
        input: PathBuf,
        start: u32,
        disable_ecc: bool,
        ignore_ecc: bool,
        strategy: BadBlockStrategy,
        oob_mode: OobMode,
        speed: Option<u8>,
    ) -> Result<()> {
        let (programmer, spec) = self.detect_use_case.execute(speed)?;
        println!("Detected chip: {} ({})", spec.name, spec.manufacturer);

        let expected_data = fs::read(input).map_err(Error::Io)?;
        let length = expected_data.len() as u32;

        println!("Verifying {} bytes starting at 0x{:08X}...", length, start);

        let params = VerifyParams {
            address: start,
            data: &expected_data,
            use_ecc: !disable_ecc,
            ignore_ecc_errors: ignore_ecc,
            oob_mode,
            bad_block_strategy: strategy,
        };

        match spec.flash_type {
            FlashType::Nand => {
                let protocol = SpiNand::new(programmer, spec);
                let mut use_case = VerifyFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    print!("\rReading for verification: {:.1}%", progress.percentage());
                    let _ = std::io::stdout().flush();
                })?
            }
            FlashType::Nor => {
                let protocol = SpiNor::new(programmer, spec);
                let mut use_case = VerifyFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    print!("\rReading for verification: {:.1}%", progress.percentage());
                    let _ = std::io::stdout().flush();
                })?
            }
            FlashType::SpiEeprom => {
                let protocol = SpiEeprom::new(programmer, spec);
                let mut use_case = VerifyFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    print!("\rReading for verification: {:.1}%", progress.percentage());
                    let _ = std::io::stdout().flush();
                })?
            }
            FlashType::I2cEeprom => {
                let protocol = I2cEeprom::new(programmer, spec);
                let mut use_case = VerifyFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    print!("\rReading for verification: {:.1}%", progress.percentage());
                    let _ = std::io::stdout().flush();
                })?
            }
            FlashType::MicrowireEeprom => {
                let protocol = MicrowireEeprom::new(programmer, spec);
                let mut use_case = VerifyFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    print!("\rReading for verification: {:.1}%", progress.percentage());
                    let _ = std::io::stdout().flush();
                })?
            }
        };

        println!("\nVerification SUCCESSFUL!");
        Ok(())
    }
}
