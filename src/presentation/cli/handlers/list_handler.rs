//! CLI Handler - List
//!
//! Lists all supported chips from the registry.

use crate::application::use_cases::detect_chip::DetectChipUseCase;
use crate::error::Result;
use crate::infrastructure::chip_database::ChipRegistry;

pub struct ListHandler {
    use_case: DetectChipUseCase,
}

impl Default for ListHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ListHandler {
    pub fn new() -> Self {
        Self {
            use_case: DetectChipUseCase::new(ChipRegistry::new()),
        }
    }

    pub fn handle(&self) -> Result<()> {
        let chips = self.use_case.list_supported_chips();

        println!("Supported Flash Chips:");
        println!(
            "{:<20} {:<20} {:<15} {:<10}",
            "Manufacturer", "Model", "Type", "Capacity"
        );
        println!("{}", "-".repeat(65));

        for chip in chips {
            println!(
                "{:<20} {:<20} {:<15} {:<10}",
                chip.manufacturer, chip.name, chip.flash_type, chip.capacity
            );
        }

        Ok(())
    }
}
