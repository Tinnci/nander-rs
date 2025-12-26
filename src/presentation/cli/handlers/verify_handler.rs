//! CLI Handler - Verify
//!
//! Handles the 'verify' command by reading flash contents and comparing with a file.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::application::use_cases::detect_chip::DetectChipUseCase;
use crate::application::use_cases::read_flash::{ReadFlashUseCase, ReadParams};
use crate::domain::{FlashType, OobMode};
use crate::error::{Error, Result};
use crate::infrastructure::chip_database::ChipRegistry;
use crate::infrastructure::flash_protocol::nand::SpiNand;
use crate::infrastructure::flash_protocol::nor::SpiNor;

pub struct VerifyHandler {
    detect_use_case: DetectChipUseCase,
}

impl VerifyHandler {
    pub fn new() -> Self {
        Self {
            detect_use_case: DetectChipUseCase::new(ChipRegistry::new()),
        }
    }

    pub fn handle(&self, input: PathBuf, start: u32, disable_ecc: bool) -> Result<()> {
        let (mut programmer, spec) = self.detect_use_case.execute()?;
        println!("Detected chip: {} ({})", spec.name, spec.manufacturer);

        let expected_data = fs::read(input).map_err(Error::Io)?;
        let length = expected_data.len() as u32;

        println!("Verifying {} bytes starting at 0x{:08X}...", length, start);

        let params = ReadParams {
            address: start,
            length,
            use_ecc: !disable_ecc,
            oob_mode: OobMode::None,
        };

        let actual_data = match spec.flash_type {
            FlashType::SpiNand => {
                let mut protocol = SpiNand::new(programmer, spec);
                let mut use_case = ReadFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    print!("\rReading for verification: {:.1}%", progress.percentage());
                    let _ = std::io::stdout().flush();
                })?
            }
            FlashType::SpiNor => {
                let mut protocol = SpiNor::new(programmer, spec);
                let mut use_case = ReadFlashUseCase::new(protocol);
                use_case.execute(params, |progress| {
                    print!("\rReading for verification: {:.1}%", progress.percentage());
                    let _ = std::io::stdout().flush();
                })?
            }
        };

        println!("\nComparing data...");
        if actual_data == expected_data {
            println!("Verification SUCCESSFUL!");
            Ok(())
        } else {
            // Find first discrepancy
            for (i, (a, e)) in actual_data.iter().zip(expected_data.iter()).enumerate() {
                if a != e {
                    return Err(Error::VerificationFailed {
                        address: start + i as u32,
                        expected: *e,
                        actual: *a,
                    });
                }
            }
            // If we got here but they aren't equal, lengths must differ
            Err(Error::InvalidParameter("Data lengths differ".to_string()))
        }
    }
}
