//! SPI NAND Flash Protocol Implementation
//!
//! This module implements the SPI NAND protocol according to infrastructure standards.

#[cfg(test)]
mod tests;

use std::time::{Duration, Instant};

use crate::domain::bad_block::BadBlockStrategy;
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

    /// Get mutable access to the programmer (for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn programmer_mut(&mut self) -> &mut P {
        &mut self.programmer
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
        // Use spi_transaction_write for single USB round-trip
        let row_addr = self.page_to_row_addr(page);
        self.programmer.spi_transaction_write(&[
            CMD_NAND_PAGE_READ,
            row_addr[0],
            row_addr[1],
            row_addr[2],
        ])?;

        // Wait for page to be loaded into cache
        self.wait_ready()?;

        // Step 2: Read from Cache (03h + column address + dummy)
        // Use optimized spi_transaction for bulk read
        let col_addr = self.column_to_addr(column);
        let cmd = [
            CMD_NAND_READ_CACHE,
            col_addr[0],
            col_addr[1],
            0x00, // Dummy byte
        ];

        // Use spi_transaction which will use bulk read for large transfers
        let data = self.programmer.spi_transaction(&cmd, len)?;
        Ok(data)
    }

    fn is_bad_block(&mut self, block: u32) -> Result<bool> {
        let page_size = self.spec.layout.page_size;
        let pages_per_block = self.spec.layout.block_size / page_size;
        let first_page = block * pages_per_block;

        // Check first byte of OOB in the first page of the block
        // SPI NAND standard: Bad block marker is usually at the first byte of spare area
        let oob = self.read_page_internal(first_page, page_size as u16, 1)?;
        if oob[0] != 0xFF {
            return Ok(true);
        }

        // Standard also suggests checking the second page
        let oob = self.read_page_internal(first_page + 1, page_size as u16, 1)?;
        if oob[0] != 0xFF {
            return Ok(true);
        }

        Ok(false)
    }

    fn check_ecc_status(&mut self, page: u32) -> Result<()> {
        let status = self.get_feature(FEATURE_STATUS)?;
        let ecc_status = status & STATUS_NAND_ECC_MASK;

        match ecc_status {
            STATUS_NAND_ECC_UNCORRECTABLE => {
                log::error!("Uncorrectable ECC error at page {}", page);
                Err(Error::EccError {
                    address: page * self.spec.layout.page_size,
                })
            }
            STATUS_NAND_ECC_CORRECTED | STATUS_NAND_ECC_CORRECTED_ALT => {
                log::warn!("Corrected ECC errors at page {}", page);
                Ok(())
            }
            _ => Ok(()),
        }
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
        let pages_per_block = self.spec.layout.block_size / page_size;
        let mut result = Vec::with_capacity(request.length as usize);
        let mut remaining = request.length as usize;

        let mut current_page = start_page;
        let mut pages_read = 0;

        while pages_read < total_pages {
            let current_block = current_page / pages_per_block;

            if request.bad_block_strategy != BadBlockStrategy::Include
                && self.is_bad_block(current_block)?
            {
                match request.bad_block_strategy {
                    BadBlockStrategy::Skip => {
                        // Skip the entire block
                        current_page = (current_block + 1) * pages_per_block;
                        continue;
                    }
                    BadBlockStrategy::Fail => {
                        return Err(Error::BadBlock {
                            block: current_block,
                        });
                    }
                    _ => {}
                }
            }

            let chunk = self.read_page_internal(current_page, col_offset, read_len_per_page)?;

            // Check ECC status if requested
            if request.use_ecc && !request.ignore_ecc_errors {
                self.check_ecc_status(current_page)?;
            } else if request.use_ecc && request.ignore_ecc_errors {
                // Just log if ignored
                if let Err(e) = self.check_ecc_status(current_page) {
                    log::debug!("Ignored ECC error: {}", e);
                }
            }

            let to_copy = remaining.min(chunk.len());
            result.extend_from_slice(&chunk[..to_copy]);
            remaining -= to_copy;
            pages_read += 1;
            current_page += 1;

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
        let pages_per_block = self.spec.layout.block_size / page_size;
        let oob_size = self.spec.layout.oob_size.unwrap_or(0);

        // Calculate write parameters based on OobMode
        let (col_offset, write_len_per_page) = match request.oob_mode {
            OobMode::None => (0u16, page_size as usize),
            OobMode::Included => (0u16, (page_size + oob_size) as usize),
            OobMode::Only => (page_size as u16, oob_size as usize),
        };

        let data_len = request.data.len();
        let total_pages = data_len.div_ceil(write_len_per_page);

        let mut current_page = start_page;
        let mut offset = 0usize;
        let mut pages_written = 0;

        while pages_written < total_pages {
            let current_block = current_page / pages_per_block;

            if request.bad_block_strategy != BadBlockStrategy::Include
                && self.is_bad_block(current_block)?
            {
                match request.bad_block_strategy {
                    BadBlockStrategy::Skip => {
                        // Skip the entire block
                        current_page = (current_block + 1) * pages_per_block;
                        continue;
                    }
                    BadBlockStrategy::Fail => {
                        return Err(Error::BadBlock {
                            block: current_block,
                        });
                    }
                    _ => {}
                }
            }

            let chunk_end = (offset + write_len_per_page).min(data_len);
            let mut page_buf = vec![0xFFu8; write_len_per_page];
            page_buf[..(chunk_end - offset)].copy_from_slice(&request.data[offset..chunk_end]);

            self.write_enable()?;

            // Program Load
            let col_addr = self.column_to_addr(col_offset);
            self.programmer.set_cs(true)?;
            self.programmer
                .spi_write(&[CMD_NAND_PROGRAM_LOAD, col_addr[0], col_addr[1]])?;
            self.programmer.spi_write(&page_buf)?;
            self.programmer.set_cs(false)?;

            // Program Execute
            let row_addr = self.page_to_row_addr(current_page);
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
                    address: current_page * page_size,
                });
            }

            on_progress(Progress::new(chunk_end as u64, data_len as u64));

            offset += write_len_per_page;
            pages_written += 1;
            current_page += 1;
        }

        if request.verify {
            // Self-verify by reading back
            let verify_req = ReadRequest {
                address: request.address,
                length: request.data.len() as u32,
                use_ecc: request.use_ecc,
                ignore_ecc_errors: request.ignore_ecc_errors,
                oob_mode: request.oob_mode,
                bad_block_strategy: request.bad_block_strategy,
            };
            let read_back = self.read(verify_req, &|_| {})?;
            if read_back != request.data {
                return Err(Error::VerificationFailed {
                    address: request.address.as_u32(), // Simplified, real verify usually finds exact byte
                    expected: 0,                       // Simplified
                    actual: 0,                         // Simplified
                });
            }
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

        let mut blocks_erased = 0;
        let mut current_block = start_block;

        while blocks_erased < total_blocks {
            if request.bad_block_strategy != BadBlockStrategy::Include
                && self.is_bad_block(current_block)?
            {
                match request.bad_block_strategy {
                    BadBlockStrategy::Skip => {
                        // Go to next block without incrementing blocks_erased count?
                        // Actually, if we skip, we usually want to erase the NEXT good block to satisfy the request.
                        current_block += 1;
                        continue;
                    }
                    BadBlockStrategy::Fail => {
                        return Err(Error::BadBlock {
                            block: current_block,
                        });
                    }
                    _ => {}
                }
            }

            let page = current_block * (block_size / page_size);

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
                return Err(Error::EraseFailed {
                    block: current_block,
                });
            }

            blocks_erased += 1;
            current_block += 1;

            on_progress(Progress::new(blocks_erased as u64, total_blocks as u64));
        }

        Ok(())
    }
}
