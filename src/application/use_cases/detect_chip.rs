//! Application Use Case - Detect Chip
//!
//! Orchestrates the process of identifying a connected flash chip.

use crate::domain::{ChipSpec, JedecId};
use crate::error::Result;
use crate::infrastructure::chip_database::ChipRegistry;
use crate::infrastructure::programmer::{self, Programmer};

pub struct DetectChipUseCase {
    registry: ChipRegistry,
}

impl DetectChipUseCase {
    pub fn new(registry: ChipRegistry) -> Self {
        Self { registry }
    }

    pub fn execute(&self, speed: Option<u8>) -> Result<(Box<dyn Programmer>, ChipSpec)> {
        // 1. Discover programmer
        let mut programmer = programmer::discover()?;

        // Apply speed if specified
        if let Some(s) = speed {
            programmer.set_speed(s)?;
        }

        // 2. Read JEDEC ID
        programmer.set_cs(true)?;
        // Standard Read ID command is 0x9F
        programmer.spi_write(&[0x9F])?;
        let id_bytes = programmer.spi_read(3)?;
        programmer.set_cs(false)?;

        let jedec = JedecId::new([id_bytes[0], id_bytes[1], id_bytes[2]]);

        // 3. Lookup in registry
        let spec = self.registry.find_by_id(jedec).ok_or_else(|| {
            crate::error::Error::UnsupportedChip(id_bytes[0], id_bytes[1], id_bytes[2])
        })?;

        Ok((programmer, spec))
    }

    pub fn list_supported_chips(&self) -> Vec<ChipSpec> {
        self.registry.list_all()
    }
}
