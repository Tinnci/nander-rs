//! CLI Handler - Protect and Status
//!
//! Handles 'protect' and 'status' commands for managing flash registers.

use crate::application::use_cases::detect_chip::DetectChipUseCase;
use crate::application::use_cases::status_flash::StatusUseCase;
use crate::domain::{FlashOperation, FlashType};
use crate::error::{Error, Result};
use crate::infrastructure::chip_database::ChipRegistry;
use crate::infrastructure::flash_protocol::eeprom::SpiEeprom;
use crate::infrastructure::flash_protocol::nand::SpiNand;
use crate::infrastructure::flash_protocol::nor::SpiNor;
use colored::*;

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

        let status_use_case: Box<dyn crate::domain::FlashOperation> = match spec.flash_type {
            FlashType::Nor => Box::new(SpiNor::new(programmer, spec)),
            FlashType::Nand => Box::new(SpiNand::new(programmer, spec)),
            FlashType::SpiEeprom => Box::new(SpiEeprom::new(programmer, spec)),
            _ => {
                return Err(Error::NotSupported(
                    "Status access not implemented for this flash type".to_string(),
                ))
            }
        };

        let mut use_case = StatusUseCase::new(status_use_case);

        if let Some(hex_val) = value {
            let bytes = hex::decode(hex_val.replace("0x", ""))
                .map_err(|e| Error::Validation(format!("Invalid hex value: {}", e)))?;

            println!("Writing status register(s): {:?}", bytes);
            use_case.set_status(&bytes)?;
            println!("{}", "SUCCESS!".green().bold());
        } else {
            let status = use_case.get_status()?;
            print!("{}", "Status register(s):".cyan().bold());
            for b in status {
                print!(" 0x{:02X}", b);
            }
            println!();
        }

        Ok(())
    }

    pub fn handle_protect(&self, operation: &str, speed: Option<u8>) -> Result<()> {
        let (programmer, spec) = self.detect_use_case.execute(speed)?;

        match spec.flash_type {
            FlashType::Nor => {
                let mut protocol = SpiNor::new(programmer, spec);
                let current_sr = protocol.read_status()?;
                let bp_mask = 0x3C; // BP0-BP3

                match operation {
                    "status" => {
                        println!("Current Status Register: 0x{:02X}", current_sr);
                        let locked = (current_sr & bp_mask) != 0;
                        if locked {
                            println!("Write Protection: {}", "ENABLED".red().bold());
                        } else {
                            println!("Write Protection: {}", "DISABLED".green().bold());
                        }
                    }
                    "enable" => {
                        println!("Enabling write protection (setting BP bits)...");
                        protocol.set_status(&[current_sr | bp_mask])?;
                        println!("{}", "Done.".green());
                    }
                    "disable" => {
                        println!("Disabling write protection (clearing BP bits)...");
                        protocol.set_status(&[current_sr & !bp_mask])?;
                        println!("{}", "Done.".green());
                    }
                    _ => {
                        return Err(Error::Validation(format!(
                            "Unknown operation: {}",
                            operation
                        )))
                    }
                }
            }
            FlashType::Nand => {
                let mut protocol = SpiNand::new(programmer, spec);
                let status = protocol.get_status()?;
                let prot_reg = status[0]; // 0xA0 is first in NAND get_status
                let bp_mask = 0x7C; // BP0, BP1, BP2, BP3, CMP, INV etc vary but 0x7C covers main ones

                match operation {
                    "status" => {
                        println!("Protection Register (0xA0): 0x{:02X}", prot_reg);
                        println!("Config Register (0xB0):     0x{:02X}", status[1]);
                        println!("Status Register (0xC0):     0x{:02X}", status[2]);
                        let locked = (prot_reg & bp_mask) != 0;
                        if locked {
                            println!("Write Protection: {}", "ENABLED".red().bold());
                        } else {
                            println!("Write Protection: {}", "DISABLED".green().bold());
                        }
                    }
                    "enable" => {
                        println!("Enabling NAND write protection...");
                        protocol.set_status(&[prot_reg | bp_mask, status[1], status[2]])?;
                        println!("{}", "Done.".green());
                    }
                    "disable" => {
                        println!("Disabling NAND write protection...");
                        protocol.set_status(&[prot_reg & !bp_mask, status[1], status[2]])?;
                        println!("{}", "Done.".green());
                    }
                    _ => {
                        return Err(Error::Validation(format!(
                            "Unknown operation: {}",
                            operation
                        )))
                    }
                }
            }
            FlashType::SpiEeprom => {
                let mut protocol = SpiEeprom::new(programmer, spec);
                let current_sr = protocol.get_status()?[0];
                let bp_mask = 0x0C; // BP0-BP1 for most 25xxx

                match operation {
                    "status" => {
                        println!("Status Register: 0x{:02X}", current_sr);
                        let locked = (current_sr & bp_mask) != 0;
                        if locked {
                            println!("Write Protection: {}", "ENABLED (BP bits set)".red().bold());
                        } else {
                            println!("Write Protection: {}", "DISABLED".green().bold());
                        }
                    }
                    "enable" => {
                        println!("Enabling EEPROM write protection...");
                        protocol.set_status(&[current_sr | bp_mask])?;
                        println!("{}", "Done.".green());
                    }
                    "disable" => {
                        println!("Disabling EEPROM write protection...");
                        protocol.set_status(&[current_sr & !bp_mask])?;
                        println!("{}", "Done.".green());
                    }
                    _ => {
                        return Err(Error::Validation(format!(
                            "Unknown operation: {}",
                            operation
                        )))
                    }
                }
            }
            _ => {
                return Err(Error::NotSupported(
                    "Protect command not supported for this flash type".to_string(),
                ))
            }
        }

        Ok(())
    }
}
