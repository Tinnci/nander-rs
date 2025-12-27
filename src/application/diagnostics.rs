//! Diagnostic and Test Commands
//!
//! This module provides tools for testing programmer functionality
//! without requiring a physical flash chip.

use crate::error::Result;
use crate::infrastructure::programmer::Programmer;
use colored::Colorize;

pub struct DiagnosticTool;

impl DiagnosticTool {
    /// Run comprehensive diagnostics on the programmer
    pub fn run_diagnostics(programmer: &mut dyn Programmer) -> Result<()> {
        println!(
            "\n{}",
            "=== CH341A Programmer Diagnostics ===".bold().cyan()
        );
        println!();

        // Test 1: Basic Communication
        println!("{}", "Test 1: Basic USB Communication".bold());
        match Self::test_basic_communication(programmer) {
            Ok(_) => println!("  {} USB communication OK", "✓".green()),
            Err(e) => {
                println!("  {} USB communication failed: {}", "✗".red(), e);
                return Err(e);
            }
        }
        println!();

        // Test 2: SPI Bus Test
        println!("{}", "Test 2: SPI Bus Status".bold());
        Self::test_spi_bus(programmer)?;
        println!();

        // Test 3: GPIO Control
        println!("{}", "Test 3: GPIO Control (LED Test)".bold());
        Self::test_gpio(programmer)?;
        println!();

        // Test 4: Read JEDEC ID
        println!("{}", "Test 4: JEDEC ID Detection".bold());
        Self::test_jedec_read(programmer)?;
        println!();

        println!("{}", "=== Diagnostics Complete ===".bold().green());
        Ok(())
    }

    fn test_basic_communication(programmer: &mut dyn Programmer) -> Result<()> {
        // Try to set CS pin high and low
        programmer.set_cs(false)?;
        programmer.set_cs(true)?;
        programmer.set_cs(false)?;
        Ok(())
    }

    fn test_spi_bus(programmer: &mut dyn Programmer) -> Result<()> {
        // Send dummy bytes and check for response
        programmer.set_cs(true)?;

        let mut rx = [0u8; 3];
        programmer.spi_transfer(&[0xFF, 0xFF, 0xFF], &mut rx)?;

        programmer.set_cs(false)?;

        println!("  SPI Response: {:02X} {:02X} {:02X}", rx[0], rx[1], rx[2]);

        if rx == [0xFF, 0xFF, 0xFF] {
            println!(
                "  {} No chip detected (all pins floating high)",
                "ℹ".yellow()
            );
            println!(
                "  {} This is NORMAL if no flash chip is connected",
                "ℹ".yellow()
            );
        } else if rx == [0x00, 0x00, 0x00] {
            println!("  {} Possible short circuit (all pins low)", "⚠".yellow());
        } else {
            println!("  {} Got response - chip may be present", "✓".green());
        }

        Ok(())
    }

    fn test_gpio(programmer: &mut dyn Programmer) -> Result<()> {
        println!("  Testing GPIO toggling (if your board has an LED, it may blink)");

        // Test GPIO pins (many CH341A boards have LEDs)
        for pin in 0..6 {
            programmer.gpio_set(pin, true)?;
            std::thread::sleep(std::time::Duration::from_millis(50));
            programmer.gpio_set(pin, false)?;
        }

        println!("  {} GPIO control OK", "✓".green());
        Ok(())
    }

    fn test_jedec_read(programmer: &mut dyn Programmer) -> Result<()> {
        // Standard JEDEC Read ID command (0x9F)
        programmer.set_cs(true)?;
        programmer.spi_write(&[0x9F])?;
        let id = programmer.spi_read(3)?;
        programmer.set_cs(false)?;

        println!("  JEDEC ID: {:02X} {:02X} {:02X}", id[0], id[1], id[2]);

        // Interpret the result
        if id == [0xFF, 0xFF, 0xFF] {
            println!("  {} No flash chip detected", "ℹ".yellow());
            println!(
                "  {} This is expected if nothing is connected to the SPI bus",
                "ℹ".yellow()
            );
            println!();
            println!("  {}", "How to test with a real chip:".bold());
            println!("    1. Connect a SPI flash chip to the CH341A");
            println!("    2. Common chips: W25Q64, GD25Q128, MX25L128, etc.");
            println!("    3. Ensure correct wiring: CS, CLK, MISO, MOSI, VCC, GND");
            println!("    4. Re-run this diagnostic");
        } else if id == [0x00, 0x00, 0x00] {
            println!("  {} All zeros - possible wiring issue", "⚠".yellow());
            println!("  {} Check connections and power supply", "⚠".yellow());
        } else {
            println!("  {} Valid chip response detected!", "✓".green());

            // Try to identify manufacturer
            let manufacturer = match id[0] {
                0xEF => "Winbond",
                0xC8 => "GigaDevice",
                0xC2 => "Macronix",
                0x20 => "Micron/XTX",
                0x1C => "EON",
                0x9D => "ISSI",
                0xBF => "SST/Microchip",
                0x01 => "Spansion/Cypress",
                _ => "Unknown",
            };

            println!("  Possible Manufacturer: {}", manufacturer);
        }

        Ok(())
    }

    /// Interactive SPI command sender
    pub fn interactive_spi_test(programmer: &mut dyn Programmer) -> Result<()> {
        println!(
            "\n{}",
            "=== Interactive SPI Command Tester ===".bold().cyan()
        );
        println!("Send raw SPI commands to test the bus");
        println!("Example: Write 0x9F, Read 3 bytes (JEDEC ID)");
        println!("Type 'quit' to exit\n");

        use std::io::{self, Write};

        loop {
            print!("Enter command (hex bytes, e.g., '9F 00 00 00'): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
                break;
            }

            // Parse hex bytes
            let bytes: std::result::Result<Vec<u8>, _> = input
                .split_whitespace()
                .map(|s| u8::from_str_radix(s, 16))
                .collect();

            match bytes {
                Ok(tx) if !tx.is_empty() => {
                    programmer.set_cs(true)?;

                    let mut rx = vec![0u8; tx.len()];
                    programmer.spi_transfer(&tx, &mut rx)?;

                    programmer.set_cs(false)?;

                    println!(
                        "  TX: {}",
                        tx.iter()
                            .map(|b| format!("{:02X}", b))
                            .collect::<Vec<_>>()
                            .join(" ")
                    );
                    println!(
                        "  RX: {}",
                        rx.iter()
                            .map(|b| format!("{:02X}", b))
                            .collect::<Vec<_>>()
                            .join(" ")
                    );
                }
                Ok(_) => println!("  Error: No bytes entered"),
                Err(e) => println!("  Error parsing hex: {}", e),
            }
            println!();
        }

        Ok(())
    }
}
