//! SPI NOR Flash Protocol Implementation
//!
//! This module implements the SPI NOR protocol according to infrastructure standards.

use std::time::{Duration, Instant};

use crate::domain::chip::ChipSpec;
use crate::domain::{EraseRequest, FlashOperation, Progress, ReadRequest, WriteRequest};
use crate::error::{Error, Result};
use crate::infrastructure::flash_protocol::commands::*;
use crate::infrastructure::programmer::Programmer;

pub struct SpiNor<P: Programmer> {
    programmer: P,
    spec: ChipSpec,
}

impl<P: Programmer> SpiNor<P> {
    pub fn new(programmer: P, spec: ChipSpec) -> Self {
        Self { programmer, spec }
    }

    /// Get chip specification
    pub fn spec(&self) -> &ChipSpec {
        &self.spec
    }

    // =========================================================================
    // Internal Helper Methods
    // =========================================================================

    fn wait_ready(&mut self) -> Result<()> {
        let start = Instant::now();
        let timeout = Duration::from_secs(30); // NOR chip erase can take longer
        loop {
            let status = self.read_status()?;
            if status & STATUS_NOR_WIP == 0 {
                return Ok(());
            }
            if start.elapsed() > timeout {
                return Err(Error::Timeout);
            }
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

    fn write_enable(&mut self) -> Result<()> {
        self.programmer.set_cs(true)?;
        self.programmer.spi_write(&[CMD_WRITE_ENABLE])?;
        self.programmer.set_cs(false)?;
        Ok(())
    }

    fn addr_to_bytes(&self, addr: u32) -> [u8; 3] {
        [(addr >> 16) as u8, (addr >> 8) as u8, addr as u8]
    }

    fn read_internal(&mut self, address: u32, len: usize) -> Result<Vec<u8>> {
        let addr_bytes = self.addr_to_bytes(address);
        self.programmer.set_cs(true)?;
        self.programmer
            .spi_write(&[CMD_NOR_READ, addr_bytes[0], addr_bytes[1], addr_bytes[2]])?;
        let data = self.programmer.spi_read(len)?;
        self.programmer.set_cs(false)?;
        Ok(data)
    }
}

impl<P: Programmer> FlashOperation for SpiNor<P> {
    fn read(&mut self, request: ReadRequest, on_progress: &dyn Fn(Progress)) -> Result<Vec<u8>> {
        // NOR flash supports arbitrary address reads
        let address = request.address.as_u32();
        let length = request.length as usize;

        // Read in chunks for progress reporting
        const CHUNK_SIZE: usize = 4096;
        let mut result = Vec::with_capacity(length);
        let mut remaining = length;
        let mut current_addr = address;

        while remaining > 0 {
            let chunk_size = remaining.min(CHUNK_SIZE);
            let chunk = self.read_internal(current_addr, chunk_size)?;
            result.extend_from_slice(&chunk);

            remaining -= chunk_size;
            current_addr += chunk_size as u32;

            on_progress(Progress::new(result.len() as u64, length as u64));
        }

        Ok(result)
    }

    fn write(&mut self, request: WriteRequest, on_progress: &dyn Fn(Progress)) -> Result<()> {
        // NOR flash must be written in 256-byte pages
        const PAGE_SIZE: usize = 256;

        let data = request.data;
        let mut offset = 0usize;
        let mut current_addr = request.address.as_u32();

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

            on_progress(Progress::new(offset as u64, data.len() as u64));
        }

        if request.verify {
            // Self-verify could be implemented here
        }

        Ok(())
    }

    fn erase(&mut self, request: EraseRequest, on_progress: &dyn Fn(Progress)) -> Result<()> {
        let block_size = self.spec.layout.block_size;
        let address = request.address.as_u32();

        if address % block_size != 0 {
            return Err(Error::InvalidParameter(
                "NOR erase address must be block-aligned".to_string(),
            ));
        }

        let total_blocks = (request.length + block_size - 1) / block_size;

        for i in 0..total_blocks {
            let block_addr = address + (i * block_size);

            self.write_enable()?;

            let addr_bytes = self.addr_to_bytes(block_addr);
            self.programmer.set_cs(true)?;
            self.programmer.spi_write(&[
                CMD_NOR_BLOCK_ERASE_64K,
                addr_bytes[0],
                addr_bytes[1],
                addr_bytes[2],
            ])?;
            self.programmer.set_cs(false)?;

            self.wait_ready()?;

            on_progress(Progress::new((i + 1) as u64, total_blocks as u64));
        }

        Ok(())
    }
}
