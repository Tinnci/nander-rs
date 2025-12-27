//! CLI Handler - Info
//!
//! Handles the 'info' command by invoking the detect chip use case.

use crate::application::use_cases::detect_chip::DetectChipUseCase;
use crate::error::Result;
use crate::infrastructure::chip_database::ChipRegistry;

pub struct InfoHandler {
    use_case: DetectChipUseCase,
}

impl Default for InfoHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl InfoHandler {
    pub fn new() -> Self {
        Self {
            use_case: DetectChipUseCase::new(ChipRegistry::new()),
        }
    }

    pub fn handle(&self, speed: Option<u8>) -> Result<()> {
        use colored::*;

        println!("Detecting flash chip...");

        match self.use_case.execute(speed) {
            Ok((programmer, spec)) => {
                println!("Programmer:   {}", programmer.name().cyan());
                println!("----------------------------------");
                println!("Manufacturer: {}", spec.manufacturer.green());
                println!("Model:        {}", spec.name.green().bold());
                println!("Type:         {}", spec.flash_type.to_string().yellow());
                println!("Capacity:     {}", spec.capacity.to_string().cyan());
                println!("ID:           {}", spec.jedec_id.to_string().blue());

                if let Some(oob) = spec.layout.oob_size {
                    println!(
                        "Page Size:    {} + {}",
                        spec.layout.page_size,
                        format!("{} OOB", oob).yellow()
                    );
                } else {
                    println!("Page Size:    {}", spec.layout.page_size);
                }

                println!("Block Size:   {}", spec.layout.block_size);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
