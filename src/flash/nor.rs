//! SPI NOR Flash implementation
//!
//! This module implements the SPI NOR protocol for standard
//! serial NOR flash chips (W25Q, MX25L, etc.)

use log::{debug, trace};
use std::time::{Duration, Instant};

use super::commands::*;
use super::{FlashOps, NorFlash};
use crate::database::ChipInfo;
use crate::error::Result;
use crate::hardware::Programmer;

/// SPI NOR Flash handler
pub struct SpiNor<P: Programmer> {
    programmer: P,
    chip: ChipInfo,
}

/// Default timeout for flash operations (30 seconds for chip erase)
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

impl<P: Programmer> SpiNor<P> {
    /// Create a new SPI NOR handler
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

    /// Issue write enable command
    fn write_enable(&mut self) -> Result<()> {
        self.programmer.set_cs(true)?;
        self.programmer.spi_write(&[CMD_WRITE_ENABLE])?;
        self.programmer.set_cs(false)?;
        Ok(())
    }

    /// Convert address to 3-byte array
    fn addr_to_bytes(&self, addr: u32) -> [u8; 3] {
        [(addr >> 16) as u8, (addr >> 8) as u8, addr as u8]
    }
}

impl<P: Programmer> FlashOps for SpiNor<P> {
    fn erase_block(&mut self, address: u32) -> Result<()> {
        debug!("Erasing 64KB block at 0x{:08X}", address);

        self.write_enable()?;

        let addr_bytes = self.addr_to_bytes(address);
        self.programmer.set_cs(true)?;
        self.programmer.spi_write(&[
            CMD_NOR_BLOCK_ERASE_64K,
            addr_bytes[0],
            addr_bytes[1],
            addr_bytes[2],
        ])?;
        self.programmer.set_cs(false)?;

        self.wait_ready()?;
        Ok(())
    }

    fn read_page(&mut self, page: u32, buffer: &mut [u8]) -> Result<()> {
        // For NOR, a "page" is typically 256 bytes
        let address = page * 256;

        let addr_bytes = self.addr_to_bytes(address);
        self.programmer.set_cs(true)?;
        self.programmer
            .spi_write(&[CMD_NOR_READ, addr_bytes[0], addr_bytes[1], addr_bytes[2]])?;

        let data = self.programmer.spi_read(buffer.len())?;
        buffer.copy_from_slice(&data);

        self.programmer.set_cs(false)?;
        Ok(())
    }

    fn write_page(&mut self, page: u32, data: &[u8]) -> Result<()> {
        // For NOR, must write in 256-byte page chunks
        let address = page * 256;

        trace!("Writing page at 0x{:08X}", address);

        self.write_enable()?;

        let addr_bytes = self.addr_to_bytes(address);
        self.programmer.set_cs(true)?;
        self.programmer.spi_write(&[
            CMD_NOR_PAGE_PROGRAM,
            addr_bytes[0],
            addr_bytes[1],
            addr_bytes[2],
        ])?;
        self.programmer.spi_write(data)?;
        self.programmer.set_cs(false)?;

        self.wait_ready()?;
        Ok(())
    }

    fn wait_ready(&mut self) -> Result<()> {
        let start = Instant::now();
        loop {
            let status = self.read_status()?;
            if status & STATUS_NOR_WIP == 0 {
                return Ok(());
            }
            if start.elapsed() > DEFAULT_TIMEOUT {
                return Err(crate::error::Error::Timeout);
            }
            // Small delay to avoid hammering the bus
            std::thread::sleep(Duration::from_micros(100));
        }
    }

    fn read_status(&mut self) -> Result<u8> {
        self.programmer.set_cs(true)?;
        self.programmer.spi_write(&[CMD_READ_STATUS_ALT])?;
        let data = self.programmer.spi_read(1)?;
        self.programmer.set_cs(false)?;
        Ok(data[0])
    }
}

impl<P: Programmer> NorFlash for SpiNor<P> {
    fn erase_sector(&mut self, address: u32) -> Result<()> {
        debug!("Erasing 4KB sector at 0x{:08X}", address);

        self.write_enable()?;

        let addr_bytes = self.addr_to_bytes(address);
        self.programmer.set_cs(true)?;
        self.programmer.spi_write(&[
            CMD_NOR_SECTOR_ERASE_4K,
            addr_bytes[0],
            addr_bytes[1],
            addr_bytes[2],
        ])?;
        self.programmer.set_cs(false)?;

        self.wait_ready()?;
        Ok(())
    }

    fn chip_erase(&mut self) -> Result<()> {
        debug!("Erasing entire chip...");

        self.write_enable()?;

        self.programmer.set_cs(true)?;
        self.programmer.spi_write(&[CMD_NOR_CHIP_ERASE])?;
        self.programmer.set_cs(false)?;

        // Chip erase can take a long time (30+ seconds for large chips)
        self.wait_ready()?;
        Ok(())
    }

    fn read(&mut self, address: u32, buffer: &mut [u8]) -> Result<()> {
        trace!("Reading {} bytes from 0x{:08X}", buffer.len(), address);

        let addr_bytes = self.addr_to_bytes(address);
        self.programmer.set_cs(true)?;
        self.programmer
            .spi_write(&[CMD_NOR_READ, addr_bytes[0], addr_bytes[1], addr_bytes[2]])?;

        let data = self.programmer.spi_read(buffer.len())?;
        buffer.copy_from_slice(&data);

        self.programmer.set_cs(false)?;
        Ok(())
    }

    fn write(&mut self, address: u32, data: &[u8]) -> Result<()> {
        // NOR flash must be written in 256-byte pages
        const PAGE_SIZE: usize = 256;

        let mut offset = 0usize;
        let mut current_addr = address;

        while offset < data.len() {
            // Calculate bytes remaining in current page
            let page_offset = (current_addr as usize) % PAGE_SIZE;
            let bytes_in_page = PAGE_SIZE - page_offset;
            let bytes_to_write = bytes_in_page.min(data.len() - offset);

            self.write_enable()?;

            let addr_bytes = self.addr_to_bytes(current_addr);
            self.programmer.set_cs(true)?;
            self.programmer.spi_write(&[
                CMD_NOR_PAGE_PROGRAM,
                addr_bytes[0],
                addr_bytes[1],
                addr_bytes[2],
            ])?;
            self.programmer
                .spi_write(&data[offset..offset + bytes_to_write])?;
            self.programmer.set_cs(false)?;

            self.wait_ready()?;

            offset += bytes_to_write;
            current_addr += bytes_to_write as u32;
        }

        Ok(())
    }
}
