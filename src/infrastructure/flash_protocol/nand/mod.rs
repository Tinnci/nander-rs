//! SPI NAND Flash Protocol Implementation
//!
//! This module implements the SPI NAND protocol according to infrastructure standards.

use std::time::{Duration, Instant};

use crate::domain::chip::ChipSpec;
use crate::domain::{EraseRequest, FlashOperation, OobMode, Progress, ReadRequest, WriteRequest};
use crate::error::{Error, Result};
use crate::infrastructure::flash_protocol::commands::*;
use crate::infrastructure::programmer::Programmer;

pub struct SpiNand<P: Programmer> {
    programmer: P,
    spec: ChipSpec,
}

impl<P: Programmer> SpiNand<P> {
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
        let timeout = Duration::from_secs(5);
        loop {
            let status = self.get_feature(FEATURE_STATUS)?;
            if status & STATUS_NAND_OIP == 0 {
                return Ok(());
            }
            if start.elapsed() > timeout {
                return Err(Error::Timeout);
            }
            std::thread::sleep(Duration::from_micros(100));
        }
    }

    fn get_feature(&mut self, addr: u8) -> Result<u8> {
        self.programmer.set_cs(true)?;
        self.programmer.spi_write(&[CMD_NAND_GET_FEATURE, addr])?;
        let data = self.programmer.spi_read(1)?;
        self.programmer.set_cs(false)?;
        Ok(data[0])
    }

    fn set_feature(&mut self, addr: u8, value: u8) -> Result<()> {
        self.programmer.set_cs(true)?;
        self.programmer
            .spi_write(&[CMD_NAND_SET_FEATURE, addr, value])?;
        self.programmer.set_cs(false)?;
        Ok(())
    }

    fn write_enable(&mut self) -> Result<()> {
        self.programmer.set_cs(true)?;
        self.programmer.spi_write(&[CMD_WRITE_ENABLE])?;
        self.programmer.set_cs(false)?;
        Ok(())
    }

    fn page_to_row_addr(&self, page: u32) -> [u8; 3] {
        [(page >> 16) as u8, (page >> 8) as u8, page as u8]
    }

    fn column_to_addr(&self, column: u16) -> [u8; 2] {
        [(column >> 8) as u8, column as u8]
    }

    fn set_ecc(&mut self, enabled: bool) -> Result<()> {
        let config = self.get_feature(FEATURE_CONFIG)?;
        if enabled {
            self.set_feature(FEATURE_CONFIG, config | CONFIG_ECC_ENABLE)
        } else {
            self.set_feature(FEATURE_CONFIG, config & !CONFIG_ECC_ENABLE)
        }
    }

    fn read_page_internal(&mut self, page: u32, column: u16, len: usize) -> Result<Vec<u8>> {
        // Step 1: Page Read to Cache (13h + row address)
        let row_addr = self.page_to_row_addr(page);
        self.programmer.set_cs(true)?;
        self.programmer
            .spi_write(&[CMD_NAND_PAGE_READ, row_addr[0], row_addr[1], row_addr[2]])?;
        self.programmer.set_cs(false)?;

        // Wait for page to be loaded into cache
        self.wait_ready()?;

        // Step 2: Read from Cache (03h/0Bh + column address + dummy)
        let col_addr = self.column_to_addr(column);
        self.programmer.set_cs(true)?;
        self.programmer.spi_write(&[
            CMD_NAND_READ_CACHE,
            col_addr[0],
            col_addr[1],
            0x00, // Dummy byte
        ])?;

        let data = self.programmer.spi_read(len)?;
        self.programmer.set_cs(false)?;
        Ok(data)
    }
}

impl<P: Programmer> FlashOperation for SpiNand<P> {
    fn read(&mut self, request: ReadRequest, on_progress: &dyn Fn(Progress)) -> Result<Vec<u8>> {
        let page_size = self.spec.layout.page_size;
        let oob_size = self.spec.layout.oob_size.unwrap_or(0);

        self.set_ecc(request.use_ecc)?;

        let start_addr = request.address.as_u32();
        let start_page = start_addr / page_size;

        // Calculate read parameters based on OobMode
        let (col_offset, read_len_per_page) = match request.oob_mode {
            OobMode::None => (0u16, page_size as usize),
            OobMode::Included => (0u16, (page_size + oob_size) as usize),
            OobMode::Only => (page_size as u16, oob_size as usize),
        };

        let total_pages = request.length.div_ceil(page_size);
        let mut result = Vec::with_capacity(request.length as usize);
        let mut remaining = request.length as usize;

        for i in 0..total_pages {
            let page = start_page + i;
            let chunk = self.read_page_internal(page, col_offset, read_len_per_page)?;

            let to_copy = remaining.min(chunk.len());
            result.extend_from_slice(&chunk[..to_copy]);
            remaining -= to_copy;

            on_progress(Progress::new(result.len() as u64, request.length as u64));

            if remaining == 0 {
                break;
            }
        }

        Ok(result)
    }

    fn write(&mut self, request: WriteRequest, on_progress: &dyn Fn(Progress)) -> Result<()> {
        let page_size = self.spec.layout.page_size;
        self.set_ecc(request.use_ecc)?;

        let start_addr = request.address.as_u32();
        if !start_addr.is_multiple_of(page_size) {
            return Err(Error::InvalidParameter(
                "NAND write address must be page-aligned".to_string(),
            ));
        }

        let start_page = start_addr / page_size;
        let data_len = request.data.len();
        let total_pages = data_len.div_ceil(page_size as usize);

        for i in 0..total_pages {
            let page = start_page + i as u32;
            let offset = i * page_size as usize;
            let chunk_end = (offset + page_size as usize).min(data_len);
            let mut page_buf = vec![0xFFu8; page_size as usize];
            page_buf[..(chunk_end - offset)].copy_from_slice(&request.data[offset..chunk_end]);

            self.write_enable()?;

            // Program Load
            let col_addr = self.column_to_addr(0);
            self.programmer.set_cs(true)?;
            self.programmer
                .spi_write(&[CMD_NAND_PROGRAM_LOAD, col_addr[0], col_addr[1]])?;
            self.programmer.spi_write(&page_buf)?;
            self.programmer.set_cs(false)?;

            // Program Execute
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

            let status = self.get_feature(FEATURE_STATUS)?;
            if status & STATUS_NAND_P_FAIL != 0 {
                return Err(Error::WriteFailed {
                    address: page * page_size,
                });
            }

            on_progress(Progress::new(chunk_end as u64, data_len as u64));
        }

        if request.verify {
            // Self-verify could be implemented here by calling read and comparing
        }

        Ok(())
    }

    fn erase(&mut self, request: EraseRequest, on_progress: &dyn Fn(Progress)) -> Result<()> {
        let block_size = self.spec.layout.block_size;
        let page_size = self.spec.layout.page_size;

        let start_addr = request.address.as_u32();
        if !start_addr.is_multiple_of(block_size) {
            return Err(Error::InvalidParameter(
                "NAND erase address must be block-aligned".to_string(),
            ));
        }

        let total_blocks = request.length.div_ceil(block_size);
        let start_block = start_addr / block_size;

        for i in 0..total_blocks {
            let block = start_block + i;
            let page = block * (block_size / page_size);

            self.write_enable()?;

            let row_addr = self.page_to_row_addr(page);
            self.programmer.set_cs(true)?;
            self.programmer.spi_write(&[
                CMD_NAND_BLOCK_ERASE,
                row_addr[0],
                row_addr[1],
                row_addr[2],
            ])?;
            self.programmer.set_cs(false)?;

            self.wait_ready()?;

            let status = self.get_feature(FEATURE_STATUS)?;
            if status & STATUS_NAND_E_FAIL != 0 {
                return Err(Error::EraseFailed { block });
            }

            on_progress(Progress::new((i + 1) as u64, total_blocks as u64));
        }

        Ok(())
    }
}
