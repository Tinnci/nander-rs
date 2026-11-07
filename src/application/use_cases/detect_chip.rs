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
        // 1. First attempt: Standard Read ID command (0x9F)
        programmer.set_cs(true)?;
        programmer.spi_write(&[0x9F])?;
        let id_bytes = programmer.spi_read(3)?;
        programmer.set_cs(false)?;

        let jedec = JedecId::new([id_bytes[0], id_bytes[1], id_bytes[2]]);

        // If direct match found (or matched via NAND shift fallback)
        if let Some(spec) = self.registry.find_by_id(jedec) {
            return Ok(spec);
        }

        // 2. Explicit probe using standard SPI NAND Read ID command (0x9F 0x00)
        programmer.set_cs(true)?;
        if programmer.spi_write(&[0x9F, 0x00]).is_ok() {
            if let Ok(nand_id) = programmer.spi_read(3) {
                programmer.set_cs(false)?;

                let jedec_nand = JedecId::new([nand_id[0], nand_id[1], nand_id[2]]);
                if let Some(spec) = self.registry.find_by_id(jedec_nand) {
                    return Ok(spec);
                }

                let jedec_nand_2b = JedecId::new([nand_id[0], nand_id[1], 0x00]);
                if let Some(spec) = self.registry.find_by_id(jedec_nand_2b) {
                    return Ok(spec);
                }

                if nand_id[0] != 0xFF && nand_id[0] != 0x00 {
                    return Err(crate::error::Error::UnsupportedChip(
                        nand_id[0], nand_id[1], nand_id[2],
                    ));
                }
            } else {
                programmer.set_cs(false)?;
            }
        } else {
            programmer.set_cs(false)?;
        }

        // 3. Return UnsupportedChip with best information
        if id_bytes[0] == 0xFF && id_bytes[1] != 0xFF {
            Err(crate::error::Error::UnsupportedChip(
                id_bytes[1], id_bytes[2], 0x00,
            ))
        } else {
            Err(crate::error::Error::UnsupportedChip(
                id_bytes[0], id_bytes[1], id_bytes[2],
            ))
        }
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
                is_dataflash: false,
            },
            capabilities: ChipCapabilities::default(),
            otp: None,
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

    #[test]
    fn test_detect_nand_chip_with_shifted_dummy_byte() {
        // Test chip: GD5F1GQ5RE (C8 41 00)
        let nand_jedec = JedecId::new([0xC8, 0x41, 0x00]);
        let nand_chip = ChipSpec {
            name: "GD5F1GQ5RE / GD5F1GQ5REYIG".to_string(),
            manufacturer: "GigaDevice".to_string(),
            jedec_id: nand_jedec,
            flash_type: FlashType::Nand,
            capacity: Capacity::bytes(128 * 1024 * 1024),
            layout: ChipLayout {
                page_size: 2048,
                block_size: 128 * 1024,
                oob_size: Some(128),
                is_dataflash: false,
            },
            capabilities: ChipCapabilities::default(),
            otp: None,
        };
        let registry = ChipRegistry::from_specs(vec![nand_chip.clone()]);
        let use_case = DetectChipUseCase::new(registry);

        // When reading an SPI NAND chip with standard 0x9F, the chip outputs 0xFF
        // during dummy byte, followed by 0xC8, 0x41: [0xFF, 0xC8, 0x41]
        let mut mock = MockProgrammer::new();
        mock.expect_reads(vec![vec![0xFF], vec![0xFF, 0xC8, 0x41]]);

        let result = use_case.identify_chip(&mut mock);
        assert!(result.is_ok(), "Expected to detect NAND chip from shifted bytes");
        let detected = result.unwrap();
        assert_eq!(detected.name, "GD5F1GQ5RE / GD5F1GQ5REYIG");
        assert_eq!(detected.jedec_id, nand_jedec);
    }
}
