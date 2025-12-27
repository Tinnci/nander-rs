//! Microwire EEPROM (93Cxx Series) Protocol Implementation
//!
//! This module implements the Microwire protocol for 93Cxx series devices
//! using bit-banging via GPIO.

use crate::domain::chip::ChipSpec;
use crate::domain::{
    bad_block::BadBlockStrategy, EraseRequest, FlashOperation, OobMode, Progress, ReadRequest,
    WriteRequest,
};
use crate::error::{Error, Result};
use crate::infrastructure::flash_protocol::commands::*;
use crate::infrastructure::programmer::ch341a::protocol::pins;
use crate::infrastructure::programmer::Programmer;

/// Microwire EEPROM protocol handler
pub struct MicrowireEeprom<P: Programmer> {
    programmer: P,
    _spec: ChipSpec,
    address_bits: u8,
    _word_size: u8, // 8 or 16
}

impl<P: Programmer> MicrowireEeprom<P> {
    pub fn new(programmer: P, spec: ChipSpec) -> Self {
        // Determine address bits and word size based on capacity
        let capacity = spec.capacity.as_bytes();
        let address_bits = match capacity {
            32 => 7,    // 93C06
            128 => 7,   // 93C46
            256 => 9,   // 93C56
            512 => 9,   // 93C66
            1024 => 11, // 93C76
            2048 => 11, // 93C86
            _ => 9,
        };

        Self {
            programmer,
            _spec: spec,
            address_bits,
            _word_size: 8,
        }
    }

    /// Pulse clock line
    fn pulse_clk(&mut self) -> Result<()> {
        self.programmer.gpio_set(pins::CLK, true)?;
        self.programmer.gpio_set(pins::CLK, false)
    }

    /// Send a single bit
    fn send_bit(&mut self, bit: bool) -> Result<()> {
        self.programmer.gpio_set(pins::DOUT, bit)?;
        self.pulse_clk()
    }

    /// Read a single bit
    fn read_bit(&mut self) -> Result<bool> {
        self.programmer.gpio_set(pins::CLK, true)?;
        let bit = self.programmer.gpio_get(pins::DIN)?;
        self.programmer.gpio_set(pins::CLK, false)?;
        Ok(bit)
    }

    /// Send bits (MSB first)
    fn send_bits(&mut self, value: u32, count: u8) -> Result<()> {
        for i in (0..count).rev() {
            let bit = (value >> i) & 1 != 0;
            self.send_bit(bit)?;
        }
        Ok(())
    }

    /// Read bits (MSB first)
    fn read_bits(&mut self, count: u8) -> Result<u32> {
        let mut value = 0u32;
        for _ in 0..count {
            let bit = self.read_bit()?;
            value = (value << 1) | (if bit { 1 } else { 0 });
        }
        Ok(value)
    }

    /// Start a transaction (CS high)
    fn start(&mut self) -> Result<()> {
        self.programmer.gpio_set(pins::CS, true)?;
        // Microwire requires a start bit (1)
        self.send_bit(true)
    }

    /// End a transaction (CS low)
    fn stop(&mut self) -> Result<()> {
        self.programmer.gpio_set(pins::CS, false)?;
        self.programmer.gpio_set(pins::DOUT, false)
    }

    /// Execute Write Enable (EWEN)
    fn write_enable(&mut self, enable: bool) -> Result<()> {
        self.start()?;
        let op = MW_OP_EWEN; // EWEN, EWDS, ERAL, WRAL all start with 100b
        self.send_bits(op as u32, 2); // Send the other 2 bits of opcode

        let addr = if enable {
            0b11 << (self.address_bits - 2)
        } else {
            0
        };
        self.send_bits(addr, self.address_bits)?;
        self.stop()
    }

    fn wait_ready(&mut self) -> Result<()> {
        self.programmer.gpio_set(pins::CS, true)?;
        let mut ready = false;
        for _ in 0..1000 {
            if self.programmer.gpio_get(pins::DIN)? {
                ready = true;
                break;
            }
        }
        self.stop()?;
        if !ready {
            return Err(Error::Timeout);
        }
        Ok(())
    }
}

impl<P: Programmer> FlashOperation for MicrowireEeprom<P> {
    fn read(&mut self, request: ReadRequest, on_progress: &dyn Fn(Progress)) -> Result<Vec<u8>> {
        let address = request.address.as_u32();
        let length = request.length as usize;
        let mut data = Vec::with_capacity(length);

        for i in 0..length {
            let curr_addr = address + i as u32;
            self.start()?;
            self.send_bits(MW_OP_READ as u32, 2)?;
            self.send_bits(curr_addr, self.address_bits)?;
            let byte = self.read_bits(8)? as u8;
            data.push(byte);
            self.stop()?;

            if i % 16 == 0 {
                on_progress(Progress::new(i as u64, length as u64));
            }
        }

        on_progress(Progress::new(length as u64, length as u64));
        Ok(data)
    }

    fn write(&mut self, request: WriteRequest, on_progress: &dyn Fn(Progress)) -> Result<()> {
        let address = request.address.as_u32();
        let data = request.data;

        self.write_enable(true)?;

        for i in 0..data.len() {
            let curr_addr = address + i as u32;
            self.start()?;
            self.send_bits(MW_OP_WRITE as u32, 2)?;
            self.send_bits(curr_addr, self.address_bits)?;
            self.send_bits(data[i] as u32, 8)?;
            self.stop()?;

            // Wait for internal write cycle
            self.wait_ready()?;

            if i % 4 == 0 {
                on_progress(Progress::new(i as u64, data.len() as u64));
            }
        }

        self.write_enable(false)?;
        on_progress(Progress::new(data.len() as u64, data.len() as u64));

        if request.verify {
            // Verification logic is usually called by and handled in the use case
        }

        Ok(())
    }

    fn erase(&mut self, request: EraseRequest, on_progress: &dyn Fn(Progress)) -> Result<()> {
        let fill_data = vec![0xFF; request.length as usize];
        let write_req = WriteRequest {
            address: request.address,
            data: &fill_data,
            use_ecc: false,
            verify: false,
            ignore_ecc_errors: true,
            bad_block_strategy: BadBlockStrategy::Fail,
            oob_mode: OobMode::None,
        };
        self.write(write_req, on_progress)
    }
}
