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

    pub fn execute(&self) -> Result<(Box<dyn Programmer>, ChipSpec)> {
        // 1. Discover programmer
        let mut programmer = programmer::discover()?;

        // 2. Read JEDEC ID
        // Note: We need a way to read JEDEC ID that's common or specialized.
        // For now, let's assume raw SPI transfer.
        programmer.set_cs(true)?;
        let mut id_raw = [0u8; 3];
        // Standard Read ID command is 0x9F
        programmer.spi_transfer(&[0x9F, 0x00, 0x00, 0x00], &mut [0u8; 4])?; // Dummy bytes might be needed

        // Wait, different chips have different JEDEC read lengths.
        // Simplified for now:
        let id_bytes = programmer.spi_read(3)?;
        programmer.set_cs(false)?;

        let jedec = JedecId::new([id_bytes[0], id_bytes[1], id_bytes[2]]);

        // 3. Lookup in registry
        let spec = self.registry.find_by_id(jedec).ok_or_else(|| {
            crate::error::Error::UnsupportedChip(id_bytes[0], id_bytes[1], id_bytes[2])
        })?;

        Ok((programmer, spec))
    }
}
