//! CLI Handler - Erase
//!
//! Handles the 'erase' command by invoking the erase flash use case.

use std::io::Write;

use crate::application::use_cases::detect_chip::DetectChipUseCase;
use crate::application::use_cases::erase_flash::{EraseFlashUseCase, EraseParams};
use crate::domain::FlashType;
use crate::error::Result;
use crate::infrastructure::chip_database::ChipRegistry;
use crate::infrastructure::flash_protocol::nand::SpiNand;
use crate::infrastructure::flash_protocol::nor::SpiNor;

pub struct EraseHandler {
    detect_use_case: DetectChipUseCase,
}

impl EraseHandler {
    pub fn new() -> Self {
        Self {
            detect_use_case: DetectChipUseCase::new(ChipRegistry::new()),
        }
    }

    pub fn handle(&self, start: u32, length: Option<u32>) -> Result<()> {
        let (mut programmer, spec) = self.detect_use_case.execute()?;
        println!("Detected chip: {} ({})", spec.name, spec.manufacturer);

        let erase_len = length.unwrap_or(spec.capacity.as_u32() - start);

        let params = EraseParams {
            address: start,
            length: erase_len,
        };

        println!("Erasing {} bytes starting at 0x{:08X}...", erase_len, start);

        match spec.flash_type {
            FlashType::SpiNand => {
                let mut protocol = SpiNand::new(programmer, spec);
                let mut use_case = EraseFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    print!("\rProgress: {:.1}%", progress.percentage());
                    let _ = std::io::stdout().flush();
                })?
            }
            FlashType::SpiNor => {
                let mut protocol = SpiNor::new(programmer, spec);
                let mut use_case = EraseFlashUseCase::new(protocol);
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
