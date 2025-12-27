//! CLI Handler - Protect and Status
//!
//! Handles 'protect' and 'status' commands for managing flash registers.

use crate::application::use_cases::detect_chip::DetectChipUseCase;
use crate::domain::{FlashOperation, FlashType};
use crate::error::{Error, Result};
use crate::infrastructure::chip_database::ChipRegistry;
use crate::infrastructure::flash_protocol::nand::SpiNand;
use crate::infrastructure::flash_protocol::nor::SpiNor;

pub struct ProtectHandler {
    detect_use_case: DetectChipUseCase,
}

impl Default for ProtectHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtectHandler {
    pub fn new() -> Self {
        Self {
            detect_use_case: DetectChipUseCase::new(ChipRegistry::new()),
        }
    }

    pub fn handle_status(&self, value: Option<String>, speed: Option<u8>) -> Result<()> {
        let (programmer, spec) = self.detect_use_case.execute(speed)?;

        let mut protocol: Box<dyn crate::domain::FlashOperation> = match spec.flash_type {
            FlashType::Nor => Box::new(SpiNor::new(programmer, spec)),
            FlashType::Nand => Box::new(SpiNand::new(programmer, spec)),
            _ => {
                return Err(Error::NotSupported(
                    "Status access not implemented for this flash type".to_string(),
                ))
            }
        };

        if let Some(hex_val) = value {
            // Write status
            let bytes = hex::decode(hex_val.replace("0x", ""))
                .map_err(|e| Error::Validation(format!("Invalid hex value: {}", e)))?;

            println!("Writing status register: {:?}", bytes);
            protocol.set_status(&bytes)?;
            println!("Done.");
        } else {
            // Read status
            let status = protocol.get_status()?;
            print!("Status register(s):");
            for b in status {
                print!(" 0x{:02X}", b);
            }
            println!();
        }

        Ok(())
    }

    pub fn handle_protect(&self, operation: &str, speed: Option<u8>) -> Result<()> {
        let (programmer, spec) = self.detect_use_case.execute(speed)?;

        if spec.flash_type != FlashType::Nor {
            return Err(Error::NotSupported(
                "Protect command currently only supports NOR flash".to_string(),
            ));
        }

        let mut protocol = SpiNor::new(programmer, spec);
        let current_sr = protocol.read_status()?;

        match operation {
            "status" => {
                println!("Current Status Register: 0x{:02X}", current_sr);
                let locked = (current_sr & 0x3C) != 0; // Check BP0-BP3 bits
                if locked {
                    println!("Write Protection: ENABLED (BP bits set)");
                } else {
                    println!("Write Protection: DISABLED (All BP bits are 0)");
                }
            }
            "enable" => {
                println!("Enabling write protection (setting all BP bits)...");
                let new_sr = current_sr | 0x3C;
                protocol.set_status(&[new_sr])?;
                println!("Done.");
            }
            "disable" => {
                println!("Disabling write protection (clearing all BP bits)...");
                let new_sr = current_sr & !0x3C;
                protocol.set_status(&[new_sr])?;
                println!("Done.");
            }
            _ => {
                return Err(Error::Validation(format!(
                    "Unknown protect operation: {}",
                    operation
                )))
            }
        }

        Ok(())
    }
}
