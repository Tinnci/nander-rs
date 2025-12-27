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

    pub fn execute(
        &self,
        speed: Option<u8>,
        driver_name: Option<&str>,
    ) -> Result<(Box<dyn Programmer>, ChipSpec)> {
        // 1. Discover programmer
        let mut programmer = programmer::discover(driver_name)?;

        // Apply speed if specified
        if let Some(s) = speed {
            programmer.set_speed(s)?;
        }

        // 2. Identify Chip
        let spec = self.identify_chip(programmer.as_mut())?;

        Ok((programmer, spec))
    }

    /// Identify the connected chip using the provided programmer
    pub fn identify_chip(&self, programmer: &mut dyn Programmer) -> Result<ChipSpec> {
        // Read JEDEC ID
        programmer.set_cs(true)?;
        // Standard Read ID command is 0x9F
        programmer.spi_write(&[0x9F])?;
        let id_bytes = programmer.spi_read(3)?;
        programmer.set_cs(false)?;

        let jedec = JedecId::new([id_bytes[0], id_bytes[1], id_bytes[2]]);

        // Lookup in registry
        let spec = self.registry.find_by_id(jedec).ok_or_else(|| {
            crate::error::Error::UnsupportedChip(id_bytes[0], id_bytes[1], id_bytes[2])
        })?;

        Ok(spec)
    }

    pub fn list_supported_chips(&self) -> Vec<ChipSpec> {
        self.registry.list_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Capacity, ChipCapabilities, ChipLayout, FlashType};
    use crate::infrastructure::programmer::mock::MockProgrammer;

    #[test]
    fn test_detect_known_chip() {
        // Setup registry with one known chip
        let known_jedec = JedecId::new([0xEF, 0x40, 0x18]);
        let known_chip = ChipSpec {
            name: "TestChip".to_string(),
            manufacturer: "TestMfg".to_string(),
            jedec_id: known_jedec,
            flash_type: FlashType::Nand,
            capacity: Capacity::bytes(1024),
            layout: ChipLayout {
                page_size: 256,
                block_size: 4096,
                oob_size: None,
            },
            capabilities: ChipCapabilities::default(),
        };
        let registry = ChipRegistry::from_specs(vec![known_chip.clone()]);
        let use_case = DetectChipUseCase::new(registry);

        // Setup mock programmer to return the ID
        let mut mock = MockProgrammer::new();
        // First response for spi_write (ignored), second for spi_read (ID)
        mock.expect_reads(vec![vec![0xFF], vec![0xEF, 0x40, 0x18]]);

        // Run detection
        let result = use_case.identify_chip(&mut mock);

        assert!(result.is_ok());
        let detected = result.unwrap();
        assert_eq!(detected.name, "TestChip");
        assert_eq!(detected.jedec_id, known_jedec);
    }

    #[test]
    fn test_detect_unknown_chip() {
        let registry = ChipRegistry::from_specs(vec![]);
        let use_case = DetectChipUseCase::new(registry);

        let mut mock = MockProgrammer::new();
        // First response for spi_write (ignored), second for spi_read (ID)
        mock.expect_reads(vec![vec![0xFF], vec![0x00, 0x01, 0x02]]);

        let result = use_case.identify_chip(&mut mock);

        assert!(result.is_err());
        match result {
            Err(crate::error::Error::UnsupportedChip(m, d, de)) => {
                assert_eq!(m, 0x00);
                assert_eq!(d, 0x01);
                assert_eq!(de, 0x02);
            }
            _ => panic!("Expected UnsupportedChip error"),
        }
    }
}
