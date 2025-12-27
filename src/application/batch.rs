//! Batch Mode - Automated Flash Programming Workflows
//!
//! Allows users to define and execute multi-step operations automatically.
//! Example: Erase → Write → Verify → Write Protect

use crate::application::use_cases::*;
use crate::domain::{BadBlockStrategy, ChipSpec, FlashType, OobMode, Progress};
use crate::error::{Error, Result};
use crate::infrastructure::chip_database::ChipRegistry;
use crate::infrastructure::flash_protocol::eeprom::{I2cEeprom, MicrowireEeprom, SpiEeprom};
use crate::infrastructure::flash_protocol::nand::SpiNand;
use crate::infrastructure::flash_protocol::nor::SpiNor;
use crate::infrastructure::programmer::Programmer;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Represents a single operation in a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BatchOperation {
    /// Detect chip and display information
    DetectChip,

    /// Erase entire chip or range
    Erase {
        #[serde(skip_serializing_if = "Option::is_none")]
        start: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        length: Option<u32>,
    },

    /// Write file to flash
    Write {
        file: PathBuf,
        #[serde(default)]
        start: u32,
        #[serde(default = "default_true")]
        verify: bool,
    },

    /// Verify flash against file
    Verify {
        file: PathBuf,
        #[serde(default)]
        start: u32,
    },

    /// Enable or disable write protection
    Protect { enable: bool },

    /// Scan for bad blocks (NAND only)
    ScanBadBlocks {
        #[serde(skip_serializing_if = "Option::is_none")]
        save_to: Option<PathBuf>,
    },

    /// Custom delay (useful for power cycling, etc.)
    Delay { milliseconds: u64 },
}

fn default_true() -> bool {
    true
}

/// A batch script containing multiple operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchScript {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub operations: Vec<BatchOperation>,
}

impl BatchScript {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            description: None,
        }
    }

    pub fn with_description(mut self, desc: String) -> Self {
        self.description = Some(desc);
        self
    }

    pub fn add_operation(mut self, op: BatchOperation) -> Self {
        self.operations.push(op);
        self
    }

    /// Load a batch script from a JSON file
    pub fn from_json_file(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| Error::Other(format!("Failed to read script file: {}", e)))?;
        let script: BatchScript = serde_json::from_str(&content)
            .map_err(|e| Error::Other(format!("Failed to parse JSON script: {}", e)))?;
        Ok(script)
    }

    /// Load a batch script from a TOML file
    pub fn from_toml_file(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| Error::Other(format!("Failed to read script file: {}", e)))?;
        let script: BatchScript = toml::from_str(&content)
            .map_err(|e| Error::Other(format!("Failed to parse TOML script: {}", e)))?;
        Ok(script)
    }

    /// Save script to JSON file
    pub fn to_json_file(&self, path: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| Error::Other(format!("Failed to serialize JSON: {}", e)))?;
        fs::write(path, content)
            .map_err(|e| Error::Other(format!("Failed to write script file: {}", e)))?;
        Ok(())
    }

    /// Save script to TOML file
    pub fn to_toml_file(&self, path: &PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| Error::Other(format!("Failed to serialize TOML: {}", e)))?;
        fs::write(path, content)
            .map_err(|e| Error::Other(format!("Failed to write script file: {}", e)))?;
        Ok(())
    }

    /// Execute all operations in sequence
    pub fn execute(
        &self,
        programmer: &mut dyn Programmer,
        registry: &ChipRegistry,
    ) -> Result<ChipSpec> {
        if let Some(desc) = &self.description {
            info!("📋 Batch: {}", desc);
        }

        info!("🚀 Executing {} operation(s)...", self.operations.len());
        info!("─────────────────────────────────────");

        // Detect chip first (required for all operations)
        let detect_use_case = DetectChipUseCase::new(registry.clone());
        let chip = detect_use_case.identify_chip(programmer)?;
        info!("✓ Detected: {} ({})", chip.name, chip.manufacturer);

        for (i, op) in self.operations.iter().enumerate() {
            info!("\n📝 Step {}/{}:", i + 1, self.operations.len());
            self.execute_operation(op, programmer, &chip)?;
        }

        info!("\n─────────────────────────────────────");
        info!("✅ Batch execution completed successfully!");
        Ok(chip)
    }

    fn execute_operation(
        &self,
        op: &BatchOperation,
        programmer: &mut dyn Programmer,
        chip: &ChipSpec,
    ) -> Result<()> {
        match op {
            BatchOperation::DetectChip => {
                info!("   ℹ Chip already detected: {}", chip.name);
                Ok(())
            }

            BatchOperation::Erase { start, length } => {
                let start_addr = start.unwrap_or(0);
                let erase_len = length.unwrap_or(chip.capacity.as_bytes() - start_addr);
                info!(
                    "   🗑️  Erasing flash (0x{:08X}, {} bytes)...",
                    start_addr, erase_len
                );

                let params = EraseParams {
                    address: start_addr,
                    length: erase_len,
                    bad_block_strategy: BadBlockStrategy::Skip,
                    bbt: None,
                };

                let on_progress = |p: Progress| {
                    if p.current.is_multiple_of(128 * 1024) || p.current == p.total {
                        info!("   [Erase] {}/{} bytes", p.current, p.total);
                    }
                };

                match chip.flash_type {
                    FlashType::Nand => {
                        EraseFlashUseCase::new(SpiNand::new(programmer, chip.clone()))
                            .execute(params, on_progress)
                    }
                    FlashType::Nor => EraseFlashUseCase::new(SpiNor::new(programmer, chip.clone()))
                        .execute(params, on_progress),
                    FlashType::SpiEeprom => {
                        EraseFlashUseCase::new(SpiEeprom::new(programmer, chip.clone()))
                            .execute(params, on_progress)
                    }
                    FlashType::I2cEeprom => {
                        EraseFlashUseCase::new(I2cEeprom::new(programmer, chip.clone()))
                            .execute(params, on_progress)
                    }
                    FlashType::MicrowireEeprom => {
                        EraseFlashUseCase::new(MicrowireEeprom::new(programmer, chip.clone()))
                            .execute(params, on_progress)
                    }
                    FlashType::SpiFram => {
                        // FRAM doesn't need erase
                        Ok(())
                    }
                }?;

                info!("   ✓ Erase complete");
                Ok(())
            }

            BatchOperation::Write {
                file,
                start,
                verify,
            } => {
                info!("   📝 Writing {:?} to 0x{:08X}...", file, start);

                let data = fs::read(file).map_err(Error::Io)?;
                info!("   📦 Data size: {} bytes", data.len());

                let params = WriteParams {
                    address: *start,
                    data: &data,
                    use_ecc: true,
                    verify: *verify,
                    ignore_ecc_errors: false,
                    oob_mode: OobMode::None,
                    bad_block_strategy: BadBlockStrategy::Skip,
                    bbt: None,
                    retry_count: 3,
                };

                let on_progress = |p: Progress| {
                    if p.current.is_multiple_of(128 * 1024) || p.current == p.total {
                        info!("   [Write] {}/{} bytes", p.current, p.total);
                    }
                };

                match chip.flash_type {
                    FlashType::Nand => {
                        WriteFlashUseCase::new(SpiNand::new(programmer, chip.clone()))
                            .execute(params, on_progress)
                    }
                    FlashType::Nor => WriteFlashUseCase::new(SpiNor::new(programmer, chip.clone()))
                        .execute(params, on_progress),
                    FlashType::SpiEeprom => {
                        WriteFlashUseCase::new(SpiEeprom::new(programmer, chip.clone()))
                            .execute(params, on_progress)
                    }
                    FlashType::I2cEeprom => {
                        WriteFlashUseCase::new(I2cEeprom::new(programmer, chip.clone()))
                            .execute(params, on_progress)
                    }
                    FlashType::MicrowireEeprom => {
                        WriteFlashUseCase::new(MicrowireEeprom::new(programmer, chip.clone()))
                            .execute(params, on_progress)
                    }
                    FlashType::SpiFram => {
                        WriteFlashUseCase::new(SpiEeprom::new(programmer, chip.clone()))
                            .execute(params, on_progress)
                    }
                }?;

                info!("   ✓ Write complete");
                Ok(())
            }

            BatchOperation::Verify { file, start } => {
                info!("   🔍 Verifying {:?} at 0x{:08X}...", file, start);

                let data = fs::read(file).map_err(Error::Io)?;

                let params = VerifyParams {
                    address: *start,
                    data: &data,
                    use_ecc: true,
                    ignore_ecc_errors: false,
                    oob_mode: OobMode::None,
                    bad_block_strategy: BadBlockStrategy::Skip,
                    bbt: None,
                    retry_count: 3,
                };

                let on_progress = |p: Progress| {
                    if p.current.is_multiple_of(128 * 1024) || p.current == p.total {
                        info!("   [Verify] {}/{} bytes", p.current, p.total);
                    }
                };

                match chip.flash_type {
                    FlashType::Nand => {
                        VerifyFlashUseCase::new(SpiNand::new(programmer, chip.clone()))
                            .execute(params, on_progress)
                    }
                    FlashType::Nor => {
                        VerifyFlashUseCase::new(SpiNor::new(programmer, chip.clone()))
                            .execute(params, on_progress)
                    }
                    FlashType::SpiEeprom => {
                        VerifyFlashUseCase::new(SpiEeprom::new(programmer, chip.clone()))
                            .execute(params, on_progress)
                    }
                    FlashType::I2cEeprom => {
                        VerifyFlashUseCase::new(I2cEeprom::new(programmer, chip.clone()))
                            .execute(params, on_progress)
                    }
                    FlashType::MicrowireEeprom => {
                        VerifyFlashUseCase::new(MicrowireEeprom::new(programmer, chip.clone()))
                            .execute(params, on_progress)
                    }
                    FlashType::SpiFram => {
                        VerifyFlashUseCase::new(SpiEeprom::new(programmer, chip.clone()))
                            .execute(params, on_progress)
                    }
                }?;

                info!("   ✓ Verification passed");
                Ok(())
            }

            BatchOperation::Protect { enable } => {
                if *enable {
                    info!("   🔒 Enabling write protection...");
                } else {
                    info!("   🔓 Disabling write protection...");
                }
                warn!("   ⚠ Write protection not yet integrated into batch executor");
                Ok(())
            }

            BatchOperation::ScanBadBlocks { save_to } => {
                info!("   🔍 Scanning for bad blocks...");
                if let Some(path) = save_to {
                    info!("   📄 Will save BBT to {:?}", path);
                }
                warn!("   ⚠ BBT scan not yet integrated into batch executor");
                Ok(())
            }

            BatchOperation::Delay { milliseconds } => {
                info!("   ⏱️  Waiting {} ms...", milliseconds);
                std::thread::sleep(std::time::Duration::from_millis(*milliseconds));
                info!("   ✓ Delay complete");
                Ok(())
            }
        }
    }
}

impl Default for BatchScript {
    fn default() -> Self {
        Self::new()
    }
}

/// Common batch script templates
pub mod templates {
    use super::*;

    /// Standard flash update: Erase → Write → Verify
    pub fn flash_update(firmware_file: PathBuf) -> BatchScript {
        BatchScript::new()
            .with_description("Flash Update (Erase → Write → Verify)".to_string())
            .add_operation(BatchOperation::Erase {
                start: None,
                length: None,
            })
            .add_operation(BatchOperation::Write {
                file: firmware_file.clone(),
                start: 0,
                verify: false,
            })
            .add_operation(BatchOperation::Verify {
                file: firmware_file,
                start: 0,
            })
    }

    /// Production programming: Erase → Write → Verify → Protect
    pub fn production_program(firmware_file: PathBuf) -> BatchScript {
        flash_update(firmware_file)
            .with_description("Production Programming (Full Workflow)".to_string())
            .add_operation(BatchOperation::Protect { enable: true })
    }
}
