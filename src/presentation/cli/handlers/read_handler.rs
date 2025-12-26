//! CLI Handler - Read
//!
//! Handles the 'read' command by invoking the read flash use case.

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::application::use_cases::detect_chip::DetectChipUseCase;
use crate::application::use_cases::read_flash::{ReadFlashUseCase, ReadParams};
use crate::domain::{BadBlockStrategy, FlashType, OobMode};
use crate::error::{Error, Result};
use crate::infrastructure::chip_database::ChipRegistry;
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

    pub fn handle(
        &self,
        output: PathBuf,
        start: u32,
        length: Option<u32>,
        disable_ecc: bool,
        strategy: BadBlockStrategy,
    ) -> Result<()> {
        let (programmer, spec) = self.detect_use_case.execute()?;
        println!("Detected chip: {} ({})", spec.name, spec.manufacturer);

        let read_len = length.unwrap_or(spec.capacity.as_bytes() - start);

        // Prepare parameters
        let params = ReadParams {
            address: start,
            length: read_len,
            use_ecc: !disable_ecc,
            oob_mode: OobMode::None, // Default to no OOB for standard read
            bad_block_strategy: strategy,
        };

        println!("Reading {} bytes starting at 0x{:08X}...", read_len, start);

        let data = match spec.flash_type {
            FlashType::Nand => {
                let protocol = SpiNand::new(programmer, spec);
                let mut use_case = ReadFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    print!("\rProgress: {:.1}%", progress.percentage());
                    let _ = std::io::stdout().flush();
                })?
            }
            FlashType::Nor => {
                let protocol = SpiNor::new(programmer, spec);
                let mut use_case = ReadFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    print!("\rProgress: {:.1}%", progress.percentage());
                    let _ = std::io::stdout().flush();
                })?
            }
        };

        println!("\nWriting to file: {:?}", output);
        let mut file = File::create(output).map_err(Error::Io)?;
        file.write_all(&data).map_err(Error::Io)?;

        println!("Done!");
        Ok(())
    }
}
