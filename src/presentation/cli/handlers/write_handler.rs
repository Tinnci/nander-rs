//! CLI Handler - Write
//!
//! Handles the 'write' command by invoking the write flash use case.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::application::use_cases::detect_chip::DetectChipUseCase;
use crate::application::use_cases::write_flash::{WriteFlashUseCase, WriteParams};
use crate::domain::FlashType;
use crate::error::{Error, Result};
use crate::infrastructure::chip_database::ChipRegistry;
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

    pub fn handle(
        &self,
        input: PathBuf,
        start: u32,
        verify: bool,
        disable_ecc: bool,
    ) -> Result<()> {
        let (programmer, spec) = self.detect_use_case.execute()?;
        println!("Detected chip: {} ({})", spec.name, spec.manufacturer);

        let data = fs::read(input).map_err(Error::Io)?;
        println!(
            "Writing {} bytes starting at 0x{:08X}...",
            data.len(),
            start
        );

        let params = WriteParams {
            address: start,
            data: &data,
            use_ecc: !disable_ecc,
            verify,
        };

        match spec.flash_type {
            FlashType::Nand => {
                let protocol = SpiNand::new(programmer, spec);
                let mut use_case = WriteFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    print!("\rProgress: {:.1}%", progress.percentage());
                    let _ = std::io::stdout().flush();
                })?
            }
            FlashType::Nor => {
                let protocol = SpiNor::new(programmer, spec);
                let mut use_case = WriteFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    print!("\rProgress: {:.1}%", progress.percentage());
                    let _ = std::io::stdout().flush();
                })?
            }
        };

        println!("\nDone!");
        Ok(())
    }
}
