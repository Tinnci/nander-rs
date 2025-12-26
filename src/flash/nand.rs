//! SPI NAND Flash implementation
//!
//! This module implements the SPI NAND protocol, including
//! page read/write with OOB handling and bad block management.

use log::{debug, trace};
use std::time::{Duration, Instant};

use super::commands::*;
use super::{FlashOps, NandFlash};
use crate::database::ChipInfo;
use crate::error::{Error, Result};
use crate::hardware::Programmer;

/// SPI NAND Flash handler
pub struct SpiNand<P: Programmer> {
    programmer: P,
    chip: ChipInfo,
}

/// Default timeout for flash operations (5 seconds)
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

impl<P: Programmer> SpiNand<P> {
    /// Create a new SPI NAND handler
    pub fn new(programmer: P, chip: ChipInfo) -> Self {
        Self { programmer, chip }
    }

    /// Get the chip information
    pub fn chip(&self) -> &ChipInfo {
        &self.chip
    }

    /// Get mutable access to the programmer
    pub fn programmer_mut(&mut self) -> &mut P {
        &mut self.programmer
    }

    /// Get feature register value
    fn get_feature(&mut self, addr: u8) -> Result<u8> {
        self.programmer.set_cs(true)?;
        self.programmer.spi_write(&[CMD_NAND_GET_FEATURE, addr])?;
        let data = self.programmer.spi_read(1)?;
        self.programmer.set_cs(false)?;
        Ok(data[0])
    }

    /// Set feature register value
    fn set_feature(&mut self, addr: u8, value: u8) -> Result<()> {
        self.programmer.set_cs(true)?;
        self.programmer
            .spi_write(&[CMD_NAND_SET_FEATURE, addr, value])?;
        self.programmer.set_cs(false)?;
        Ok(())
    }

    /// Issue write enable command
    fn write_enable(&mut self) -> Result<()> {
        self.programmer.set_cs(true)?;
        self.programmer.spi_write(&[CMD_WRITE_ENABLE])?;
        self.programmer.set_cs(false)?;
        Ok(())
    }

    /// Convert page number to row address bytes
    fn page_to_row_addr(&self, page: u32) -> [u8; 3] {
        [(page >> 16) as u8, (page >> 8) as u8, page as u8]
    }

    /// Convert column offset to column address bytes
    fn column_to_addr(&self, column: u16) -> [u8; 2] {
        [(column >> 8) as u8, column as u8]
    }

    // =========================================================================
    // Public ECC Control Methods
    // =========================================================================

    /// Check if internal ECC is currently enabled
    pub fn is_ecc_enabled(&mut self) -> Result<bool> {
        let config = self.get_feature(FEATURE_CONFIG)?;
        Ok(config & CONFIG_ECC_ENABLE != 0)
    }

    /// Enable internal ECC
    ///
    /// When ECC is enabled, the chip will automatically correct errors during read
    /// and write operations. This is the default mode for most chips.
    pub fn enable_ecc(&mut self) -> Result<()> {
        debug!("Enabling internal ECC");
        let config = self.get_feature(FEATURE_CONFIG)?;
        self.set_feature(FEATURE_CONFIG, config | CONFIG_ECC_ENABLE)?;
        debug!(
            "ECC enabled, config register: 0x{:02X}",
            config | CONFIG_ECC_ENABLE
        );
        Ok(())
    }

    /// Disable internal ECC
    ///
    /// When ECC is disabled, raw page data can be read/written including the ECC bytes.
    /// This is useful for:
    /// - Reading/writing raw images with external ECC
    /// - Backing up the complete flash content including ECC area
    /// - Debugging ECC-related issues
    pub fn disable_ecc(&mut self) -> Result<()> {
        debug!("Disabling internal ECC");
        let config = self.get_feature(FEATURE_CONFIG)?;
        self.set_feature(FEATURE_CONFIG, config & !CONFIG_ECC_ENABLE)?;
        debug!(
            "ECC disabled, config register: 0x{:02X}",
            config & !CONFIG_ECC_ENABLE
        );
        Ok(())
    }

    /// Get the current configuration register value
    pub fn get_config(&mut self) -> Result<u8> {
        self.get_feature(FEATURE_CONFIG)
    }

    /// Get status register values for diagnostics
    ///
    /// Returns (status_register_1, status_register_2) as shown in SNANDer output
    pub fn get_status_registers(&mut self) -> Result<(u8, u8)> {
        let status = self.get_feature(FEATURE_STATUS)?;
        let config = self.get_feature(FEATURE_CONFIG)?;
        Ok((status, config))
    }
}

impl<P: Programmer> FlashOps for SpiNand<P> {
    fn erase_block(&mut self, address: u32) -> Result<()> {
        let block = address / self.chip.block_size.unwrap_or(0x20000);
        let page = block * (self.chip.block_size.unwrap_or(0x20000) / self.chip.page_size);

        debug!("Erasing block {} (page {})", block, page);

        self.write_enable()?;

        // Send block erase command with row address
        let row_addr = self.page_to_row_addr(page);
        self.programmer.set_cs(true)?;
        self.programmer.spi_write(&[
            CMD_NAND_BLOCK_ERASE,
            row_addr[0],
            row_addr[1],
            row_addr[2],
        ])?;
        self.programmer.set_cs(false)?;

        // Wait for completion
        self.wait_ready()?;

        // Check for erase failure
        let status = self.read_status()?;
        if status & STATUS_NAND_E_FAIL != 0 {
            return Err(Error::EraseFailed { block });
        }

        Ok(())
    }

    fn read_page(&mut self, page: u32, buffer: &mut [u8]) -> Result<()> {
        trace!("Reading page {}", page);

        // Step 1: Page Read to Cache (13h + row address)
        let row_addr = self.page_to_row_addr(page);
        self.programmer.set_cs(true)?;
        self.programmer
            .spi_write(&[CMD_NAND_PAGE_READ, row_addr[0], row_addr[1], row_addr[2]])?;
        self.programmer.set_cs(false)?;

        // Wait for page to be loaded into cache
        self.wait_ready()?;

        // Step 2: Read from Cache (03h/0Bh + column address + data)
        let col_addr = self.column_to_addr(0);
        self.programmer.set_cs(true)?;
        self.programmer.spi_write(&[
            CMD_NAND_READ_CACHE,
            col_addr[0],
            col_addr[1],
            0x00, // Dummy byte
        ])?;

        // Read the actual data
        let data = self.programmer.spi_read(buffer.len())?;
        buffer.copy_from_slice(&data);

        self.programmer.set_cs(false)?;

        Ok(())
    }

    fn write_page(&mut self, page: u32, data: &[u8]) -> Result<()> {
        trace!("Writing page {}", page);

        self.write_enable()?;

        // Step 1: Program Load (02h + column address + data)
        let col_addr = self.column_to_addr(0);
        self.programmer.set_cs(true)?;
        self.programmer
            .spi_write(&[CMD_NAND_PROGRAM_LOAD, col_addr[0], col_addr[1]])?;
        self.programmer.spi_write(data)?;
        self.programmer.set_cs(false)?;

        // Step 2: Program Execute (10h + row address)
        let row_addr = self.page_to_row_addr(page);
        self.programmer.set_cs(true)?;
        self.programmer.spi_write(&[
            CMD_NAND_PROGRAM_EXECUTE,
            row_addr[0],
            row_addr[1],
            row_addr[2],
        ])?;
        self.programmer.set_cs(false)?;

        // Wait for programming to complete
        self.wait_ready()?;

        // Check for program failure
        let status = self.read_status()?;
        if status & STATUS_NAND_P_FAIL != 0 {
            let address = page * self.chip.page_size;
            return Err(Error::WriteFailed { address });
        }

        Ok(())
    }

    fn wait_ready(&mut self) -> Result<()> {
        // Poll status register until OIP bit clears
        let start = Instant::now();
        loop {
            let status = self.get_feature(FEATURE_STATUS)?;
            if status & STATUS_NAND_OIP == 0 {
                return Ok(());
            }
            if start.elapsed() > DEFAULT_TIMEOUT {
                return Err(Error::Timeout);
            }
            // Small delay to avoid hammering the bus
            std::thread::sleep(Duration::from_micros(100));
        }
    }

    fn read_status(&mut self) -> Result<u8> {
        self.get_feature(FEATURE_STATUS)
    }
}

impl<P: Programmer> NandFlash for SpiNand<P> {
    fn read_page_with_oob(&mut self, page: u32, data: &mut [u8], oob: &mut [u8]) -> Result<()> {
        // Read page data first
        self.read_page(page, data)?;

        // Then read OOB area (starts after page data)
        let oob_start = self.chip.page_size as u16;
        let col_addr = self.column_to_addr(oob_start);

        self.programmer.set_cs(true)?;
        self.programmer.spi_write(&[
            CMD_NAND_READ_CACHE,
            col_addr[0],
            col_addr[1],
            0x00, // Dummy byte
        ])?;

        let oob_data = self.programmer.spi_read(oob.len())?;
        oob.copy_from_slice(&oob_data);

        self.programmer.set_cs(false)?;

        Ok(())
    }

    fn write_page_with_oob(&mut self, page: u32, data: &[u8], oob: &[u8]) -> Result<()> {
        self.write_enable()?;

        // Load page data
        let col_addr = self.column_to_addr(0);
        self.programmer.set_cs(true)?;
        self.programmer
            .spi_write(&[CMD_NAND_PROGRAM_LOAD, col_addr[0], col_addr[1]])?;
        self.programmer.spi_write(data)?;
        self.programmer.set_cs(false)?;

        // Load OOB data using random data input
        let oob_start = self.chip.page_size as u16;
        let oob_col_addr = self.column_to_addr(oob_start);
        self.programmer.set_cs(true)?;
        self.programmer.spi_write(&[
            CMD_NAND_PROGRAM_LOAD_RANDOM,
            oob_col_addr[0],
            oob_col_addr[1],
        ])?;
        self.programmer.spi_write(oob)?;
        self.programmer.set_cs(false)?;

        // Execute program
        let row_addr = self.page_to_row_addr(page);
        self.programmer.set_cs(true)?;
        self.programmer.spi_write(&[
            CMD_NAND_PROGRAM_EXECUTE,
            row_addr[0],
            row_addr[1],
            row_addr[2],
        ])?;
        self.programmer.set_cs(false)?;

        self.wait_ready()?;

        // Check for failure
        let status = self.read_status()?;
        if status & STATUS_NAND_P_FAIL != 0 {
            let address = page * self.chip.page_size;
            return Err(Error::WriteFailed { address });
        }

        Ok(())
    }

    fn is_bad_block(&mut self, block: u32) -> Result<bool> {
        // Read first page of block and check bad block marker in OOB
        let pages_per_block = self.chip.block_size.unwrap_or(0x20000) / self.chip.page_size;
        let first_page = block * pages_per_block;

        let mut oob = vec![0u8; self.chip.oob_size.unwrap_or(64) as usize];
        let mut page_data = vec![0u8; self.chip.page_size as usize];

        self.read_page_with_oob(first_page, &mut page_data, &mut oob)?;

        // Bad block marker is typically at first byte of OOB
        // 0xFF = good block, anything else = bad block
        Ok(oob[0] != 0xFF)
    }

    fn mark_bad_block(&mut self, block: u32) -> Result<()> {
        let pages_per_block = self.chip.block_size.unwrap_or(0x20000) / self.chip.page_size;
        let first_page = block * pages_per_block;

        // Write 0x00 to first byte of OOB to mark as bad
        let page_data = vec![0xFF; self.chip.page_size as usize];
        let mut oob = vec![0xFF; self.chip.oob_size.unwrap_or(64) as usize];
        oob[0] = 0x00; // Bad block marker

        self.write_page_with_oob(first_page, &page_data, &oob)?;

        Ok(())
    }
}
