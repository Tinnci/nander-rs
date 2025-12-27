//! CLI Handler - Passthrough
//!
//! Handles the 'passthrough' command for raw SPI/I2C communication.

use crate::error::{Error, Result};
use crate::infrastructure::programmer;
use hex;

pub struct PassthroughHandler;

impl Default for PassthroughHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl PassthroughHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn handle(
        &self,
        driver: Option<&str>,
        speed: Option<u8>,
        mode: &str,
        tx: Option<String>,
        rx_len: usize,
        addr: Option<String>,
    ) -> Result<()> {
        use colored::*;

        // 1. Initialize Programmer
        let mut prog = programmer::discover(driver)?;
        if let Some(s) = speed {
            prog.set_speed(s)?;
        }

        // 2. Parse Inputs
        let tx_bytes = if let Some(hex_str) = tx {
            hex::decode(hex_str.replace("0x", "").replace(" ", ""))
                .map_err(|e| Error::InvalidParameter(format!("Invalid hex string: {}", e)))?
        } else {
            Vec::new()
        };

        // 3. Execute based on Mode
        match mode.to_lowercase().as_str() {
            "spi" => {
                println!(
                    "SPI Passthrough: TX={} bytes, RX={} bytes",
                    tx_bytes.len(),
                    rx_len
                );

                let rx_bytes = if !tx_bytes.is_empty() {
                    // Transaction (Write then Read, or Full Duplex?)
                    // spi_transaction usually does Write then Read.
                    // spi_transfer does Full Duplex.
                    // "Passthrough" usually implies full control.
                    // But our args are separate TX and RX len.
                    // If we use spi_transfer, we need TX len == RX len usually?
                    // Or we pad TX/RX?
                    // Programmer trait has:
                    // - spi_transfer(tx, rx) -> synchronous, len = min(tx, rx) or max?
                    // - spi_transaction(tx, rx_len) -> CS low -> Write TX -> Read RX -> CS high. (Sequential)

                    if rx_len > 0 {
                        // Write then Read (Sequential)
                        prog.spi_transaction(&tx_bytes, rx_len)?
                    } else {
                        // Write only
                        prog.spi_write(&tx_bytes)?;
                        Vec::new()
                    }
                } else if rx_len > 0 {
                    // Read only
                    prog.spi_read(rx_len)?
                } else {
                    println!("Nothing to do.");
                    return Ok(());
                };

                if !rx_bytes.is_empty() {
                    println!("RX: {}", hex::encode_upper(&rx_bytes).green());
                }
            }
            "i2c" => {
                let addr_str = addr.ok_or_else(|| {
                    Error::InvalidParameter("I2C Address required for I2C mode".to_string())
                })?;

                let addr_val = if let Some(stripped) = addr_str.strip_prefix("0x") {
                    u8::from_str_radix(stripped, 16)
                } else {
                    addr_str.parse::<u8>()
                }
                .map_err(|_| Error::InvalidParameter("Invalid I2C address".to_string()))?;

                println!(
                    "I2C Passthrough (Addr: 0x{:02X}): TX={} bytes, RX={} bytes",
                    addr_val,
                    tx_bytes.len(),
                    rx_len
                );

                if !tx_bytes.is_empty() {
                    prog.i2c_write(addr_val, &tx_bytes)?;
                }

                if rx_len > 0 {
                    let rx_bytes = prog.i2c_read(addr_val, rx_len)?;
                    println!("RX: {}", hex::encode_upper(&rx_bytes).green());
                }
            }
            _ => {
                return Err(Error::InvalidParameter(format!(
                    "Unknown mode: '{}'. Supported: spi, i2c",
                    mode
                )));
            }
        }

        Ok(())
    }
}
