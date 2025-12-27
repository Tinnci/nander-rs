//! SPI EEPROM (25xxx Series) Protocol Implementation
//!
//! This module implements the SPI EEPROM protocol for 25xxx series devices.
//! These EEPROMs use a simplified SPI Flash command set with smaller page sizes.
//!
//! # Supported Chips
//!
//! - 25010 (128 bytes)
//! - 25020 (256 bytes)
//! - 25040 (512 bytes)
//! - 25080 (1 KB)
//! - 25160 (2 KB)
//! - 25320 (4 KB)
//! - 25640 (8 KB)
//! - 25128 (16 KB)
//! - 25256 (32 KB)
//! - 25512 (64 KB)
//! - 251024 (128 KB)
//!
//! # Address Modes
//!
//! - Devices ≤512 bytes (25010-25040): 1-byte address
//! - Devices 1KB-64KB (25080-25512): 2-byte address  
//! - Devices ≥128KB (251024): 3-byte address

use std::time::{Duration, Instant};

use crate::domain::chip::ChipSpec;
use crate::domain::{EraseRequest, FlashOperation, Progress, ReadRequest, WriteRequest};
use crate::error::{Error, Result};
use crate::infrastructure::flash_protocol::commands::*;
use crate::infrastructure::programmer::Programmer;

/// Address mode for SPI EEPROM based on capacity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressMode {
    /// 1-byte address for devices ≤512 bytes
    OneByte,
    /// 2-byte address for devices 1KB-64KB
    TwoByte,
    /// 3-byte address for devices ≥128KB
    ThreeByte,
}

impl AddressMode {
    /// Determine address mode from chip capacity
    pub fn from_capacity(capacity_bytes: u32) -> Self {
        if capacity_bytes <= 512 {
            AddressMode::OneByte
        } else if capacity_bytes <= 65536 {
            AddressMode::TwoByte
        } else {
            AddressMode::ThreeByte
        }
    }

    /// Get number of address bytes
    pub fn address_bytes(&self) -> usize {
        match self {
            AddressMode::OneByte => 1,
            AddressMode::TwoByte => 2,
            AddressMode::ThreeByte => 3,
        }
    }
}

/// SPI EEPROM protocol handler
pub struct SpiEeprom<P: Programmer> {
    programmer: P,
    spec: ChipSpec,
    address_mode: AddressMode,
}

impl<P: Programmer> SpiEeprom<P> {
    pub fn new(programmer: P, spec: ChipSpec) -> Self {
        let address_mode = AddressMode::from_capacity(spec.capacity.as_bytes());
        Self {
            programmer,
            spec,
            address_mode,
        }
    }

    /// Get chip specification
    pub fn spec(&self) -> &ChipSpec {
        &self.spec
    }

    /// Get address mode
    pub fn address_mode(&self) -> AddressMode {
        self.address_mode
    }

    // =========================================================================
    // Internal Helper Methods
    // =========================================================================

    fn wait_ready(&mut self) -> Result<()> {
        let start = Instant::now();
        let timeout = Duration::from_millis(100); // EEPROM write is typically <5ms

        loop {
            let status = self.read_status()?;
            if status & STATUS_EEPROM_WIP == 0 {
                return Ok(());
            }
            if start.elapsed() > timeout {
                return Err(Error::Timeout);
            }
            std::thread::sleep(Duration::from_micros(50));
        }
    }

    fn read_status(&mut self) -> Result<u8> {
        self.programmer.set_cs(true)?;
        self.programmer.spi_write(&[CMD_EEPROM_RDSR])?;
        let data = self.programmer.spi_read(1)?;
        self.programmer.set_cs(false)?;
        Ok(data[0])
    }

    fn write_enable(&mut self) -> Result<()> {
        self.programmer.set_cs(true)?;
        self.programmer.spi_write(&[CMD_EEPROM_WREN])?;
        self.programmer.set_cs(false)?;
        Ok(())
    }

    /// Convert address to bytes and return a command mask (for bits embedded in command)
    fn addr_to_bytes(&self, addr: u32) -> (u8, Vec<u8>) {
        match self.address_mode {
            AddressMode::OneByte => {
                // For devices with 512 bytes (like 25040), bit 3 of the command byte
                // is sometimes used as the 9th address bit (A8).
                let cmd_mask = if addr >= 256 { 0x08 } else { 0x00 };
                (cmd_mask, vec![addr as u8])
            }
            AddressMode::TwoByte => (0x00, vec![(addr >> 8) as u8, addr as u8]),
            AddressMode::ThreeByte => (
                0x00,
                vec![(addr >> 16) as u8, (addr >> 8) as u8, addr as u8],
            ),
        }
    }

    /// Get page size for this EEPROM
    fn page_size(&self) -> usize {
        self.spec.layout.page_size as usize
    }
}

impl<P: Programmer> FlashOperation for SpiEeprom<P> {
    fn read(&mut self, request: ReadRequest, on_progress: &dyn Fn(Progress)) -> Result<Vec<u8>> {
        let address = request.address.as_u32();
        let length = request.length as usize;

        // For large reads, we split into smaller chunks for progress reporting
        let chunk_size = self.programmer.max_bulk_transfer_size().min(4096);
        let mut result = Vec::with_capacity(length);
        let mut remaining = length;
        let mut current_offset = 0u32;

        while remaining > 0 {
            let read_size = remaining.min(chunk_size);
            let current_addr = address + current_offset;
            let (cmd_mask, addr_bytes) = self.addr_to_bytes(current_addr);
            let mut cmd = vec![CMD_EEPROM_READ | cmd_mask];
            cmd.extend(addr_bytes);

            let mut attempts = 0;
            let chunk = loop {
                match self.programmer.spi_transaction(&cmd, read_size) {
                    Ok(data) => break data,
                    Err(e) => {
                        if attempts < request.retry_count {
                            attempts += 1;
                            log::warn!(
                                "Read error at 0x{:08X}, retrying (attempt {}): {}",
                                current_addr,
                                attempts,
                                e
                            );
                            continue;
                        } else {
                            return Err(e);
                        }
                    }
                }
            };
            result.extend_from_slice(&chunk);

            remaining -= read_size;
            current_offset += read_size as u32;

            on_progress(Progress::new(result.len() as u64, length as u64));
        }

        Ok(result)
    }

    fn write(&mut self, request: WriteRequest, on_progress: &dyn Fn(Progress)) -> Result<()> {
        let data = request.data;
        let page_size = self.page_size();
        let mut offset = 0usize;
        let mut current_addr = request.address.as_u32();

        while offset < data.len() {
            // Calculate bytes remaining in current page
            let page_offset = (current_addr as usize) % page_size;
            let bytes_in_page = page_size - page_offset;
            let bytes_to_write = bytes_in_page.min(data.len() - offset);

            self.write_enable()?;
            let (cmd_mask, addr_bytes) = self.addr_to_bytes(current_addr);

            // Build write command: CMD + Address + Data
            let mut cmd = vec![CMD_EEPROM_WRITE | cmd_mask];
            cmd.extend(addr_bytes);
            cmd.extend_from_slice(&data[offset..offset + bytes_to_write]);

            self.programmer.spi_transaction_write(&cmd)?;
            self.wait_ready()?;

            offset += bytes_to_write;
            current_addr += bytes_to_write as u32;

            on_progress(Progress::new(offset as u64, data.len() as u64));
        }

        if request.verify {
            let verify_req = ReadRequest {
                address: request.address,
                length: request.data.len() as u32,
                use_ecc: request.use_ecc,
                ignore_ecc_errors: request.ignore_ecc_errors,
                oob_mode: request.oob_mode,
                bad_block_strategy: request.bad_block_strategy,
                bbt: None,
                retry_count: request.retry_count,
            };
            let read_back = self.read(verify_req, &|_| {})?;
            if read_back != request.data {
                for (i, (&actual, &expected)) in
                    read_back.iter().zip(request.data.iter()).enumerate()
                {
                    if actual != expected {
                        return Err(Error::VerificationFailed {
                            address: request.address.as_u32() + i as u32,
                            expected,
                            actual,
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn erase(&mut self, _request: EraseRequest, on_progress: &dyn Fn(Progress)) -> Result<()> {
        // SPI EEPROMs don't require explicit erase before write
        // They can be overwritten directly
        // For a "full erase", we write 0xFF to all bytes
        let capacity = self.spec.capacity.as_bytes() as usize;
        let page_size = self.page_size();
        let fill_data = vec![0xFF; page_size];

        let mut offset = 0usize;

        while offset < capacity {
            let bytes_to_write = page_size.min(capacity - offset);

            self.write_enable()?;
            let (cmd_mask, addr_bytes) = self.addr_to_bytes(offset as u32);

            let mut cmd = vec![CMD_EEPROM_WRITE | cmd_mask];
            cmd.extend(addr_bytes);
            cmd.extend_from_slice(&fill_data[..bytes_to_write]);

            self.programmer.spi_transaction_write(&cmd)?;
            self.wait_ready()?;

            offset += bytes_to_write;
            on_progress(Progress::new(offset as u64, capacity as u64));
        }

        Ok(())
    }

    fn get_status(&mut self) -> Result<Vec<u8>> {
        Ok(vec![self.read_status()?])
    }

    fn set_status(&mut self, status: &[u8]) -> Result<()> {
        if status.is_empty() {
            return Ok(());
        }
        self.write_enable()?;
        self.programmer
            .spi_transaction_write(&[CMD_EEPROM_WRSR, status[0]])?;
        self.wait_ready()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_mode_from_capacity() {
        // 1-byte address devices
        assert_eq!(AddressMode::from_capacity(128), AddressMode::OneByte);
        assert_eq!(AddressMode::from_capacity(256), AddressMode::OneByte);
        assert_eq!(AddressMode::from_capacity(512), AddressMode::OneByte);

        // 2-byte address devices
        assert_eq!(AddressMode::from_capacity(1024), AddressMode::TwoByte);
        assert_eq!(AddressMode::from_capacity(2048), AddressMode::TwoByte);
        assert_eq!(AddressMode::from_capacity(65536), AddressMode::TwoByte);

        // 3-byte address devices
        assert_eq!(AddressMode::from_capacity(131072), AddressMode::ThreeByte);
    }

    #[test]
    fn test_address_bytes() {
        assert_eq!(AddressMode::OneByte.address_bytes(), 1);
        assert_eq!(AddressMode::TwoByte.address_bytes(), 2);
        assert_eq!(AddressMode::ThreeByte.address_bytes(), 3);
    }
}
