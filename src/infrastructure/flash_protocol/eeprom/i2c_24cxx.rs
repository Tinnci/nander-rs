//! I2C EEPROM (24Cxx Series) Protocol Implementation
//!
//! This module implements the I2C EEPROM protocol for 24Cxx series devices.
//! These EEPROMs use I2C for communication and have varying addressing modes.

use crate::domain::chip::ChipSpec;
use crate::domain::{EraseRequest, FlashOperation, Progress, ReadRequest, WriteRequest};
use crate::error::{Error, Result};
use crate::infrastructure::flash_protocol::commands::*;
use crate::infrastructure::programmer::Programmer;

/// I2C EEPROM protocol handler
pub struct I2cEeprom<P: Programmer> {
    programmer: P,
    spec: ChipSpec,
}

impl<P: Programmer> I2cEeprom<P> {
    pub fn new(programmer: P, spec: ChipSpec) -> Self {
        Self { programmer, spec }
    }

    /// Determine addressing mode based on capacity
    /// Returns (device_addr_mask, num_addr_bytes)
    fn get_addressing_info(&self, address: u32) -> (u8, Vec<u8>) {
        let capacity = self.spec.capacity.as_bytes();
        let base_addr = I2C_ADDR_24CXX;

        if capacity <= 2048 {
            // 24C01 to 24C16: 1-byte address + address bits in device ID
            let addr_high = (address >> 8) as u8 & 0x07;
            let device_addr = base_addr | (addr_high << 1);
            let addr_bytes = vec![address as u8];
            (device_addr, addr_bytes)
        } else {
            // 24C32 to 24C1024: 2-byte address
            let device_addr = base_addr;
            let addr_bytes = vec![(address >> 8) as u8, address as u8];
            (device_addr, addr_bytes)
        }
    }

    /// Get page size for this EEPROM
    fn page_size(&self) -> usize {
        self.spec.layout.page_size as usize
    }
}

impl<P: Programmer> FlashOperation for I2cEeprom<P> {
    fn read(&mut self, request: ReadRequest, on_progress: &dyn Fn(Progress)) -> Result<Vec<u8>> {
        let address = request.address.as_u32();
        let length = request.length as usize;
        let mut result = Vec::with_capacity(length);

        // CH341A I2C read is limited to ~32 bytes per transaction in my current implementation
        const MAX_I2C_READ: usize = 32;

        let mut remaining = length;
        let mut current_addr = address;

        while remaining > 0 {
            let read_size = remaining.min(MAX_I2C_READ);
            let (device_addr, addr_bytes) = self.get_addressing_info(current_addr);

            // Step 1 & 2: Write memory address and read data with retry
            let mut attempts = 0;
            let chunk = loop {
                let res = (|| -> Result<Vec<u8>> {
                    self.programmer.i2c_write(device_addr, &addr_bytes)?;
                    self.programmer.i2c_read(device_addr, read_size)
                })();

                match res {
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
            current_addr += read_size as u32;

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

            let (device_addr, addr_bytes) = self.get_addressing_info(current_addr);

            // Build I2C write packet: MemAddr + Data
            let mut packet = Vec::with_capacity(addr_bytes.len() + bytes_to_write);
            packet.extend_from_slice(&addr_bytes);
            packet.extend_from_slice(&data[offset..offset + bytes_to_write]);

            self.programmer.i2c_write(device_addr, &packet)?;

            // EEPROM write cycle time (typically 5-10ms)
            std::thread::sleep(std::time::Duration::from_millis(10));

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
                return Err(Error::VerificationFailed {
                    address: request.address.as_u32(),
                    expected: 0,
                    actual: 0,
                });
            }
        }

        Ok(())
    }

    fn erase(&mut self, _request: EraseRequest, on_progress: &dyn Fn(Progress)) -> Result<()> {
        // I2C EEPROMs are byte-writable/overwritable
        // "Full erase" means filling with 0xFF
        let capacity = self.spec.capacity.as_bytes() as usize;
        let page_size = self.page_size();
        let fill_data = vec![0xFF; page_size];

        let mut offset = 0usize;

        while offset < capacity {
            let bytes_to_write = page_size.min(capacity - offset);
            let current_addr = offset as u32;

            let (device_addr, addr_bytes) = self.get_addressing_info(current_addr);

            let mut packet = Vec::with_capacity(addr_bytes.len() + bytes_to_write);
            packet.extend_from_slice(&addr_bytes);
            packet.extend_from_slice(&fill_data[..bytes_to_write]);

            self.programmer.i2c_write(device_addr, &packet)?;
            std::thread::sleep(std::time::Duration::from_millis(10));

            offset += bytes_to_write;
            on_progress(Progress::new(offset as u64, capacity as u64));
        }

        Ok(())
    }
}
