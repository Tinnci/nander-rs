//! SPI Flash Simulator
//!
//! A high-level simulator for SPI Flash chips (NAND/NOR) to enable
//! end-to-end integration testing without hardware.

use crate::error::Result;
use crate::infrastructure::programmer::Programmer;
use std::cell::RefCell;

/// Represents the internal state of a simulated SPI NAND chip
#[derive(Debug, Clone)]
struct SpiNandState {
    /// Main storage array (flat byte vector)
    memory: Vec<u8>,
    /// Page Data Buffer (for Read/Program)
    page_buffer: Vec<u8>,
    /// Status Register (Register C0h)
    status_register: u8,
    /// Configuration Register
    #[allow(dead_code)]
    config_register: u8,
    /// Write Enable Latch
    write_enabled: bool,
    /// Current internal operation address (Row Address)
    current_row_addr: u32,
    /// Current column address pointer
    column_ptr: u16,
    /// Page Size
    page_size: u32,
    /// Block Size
    block_size: u32,
}

impl SpiNandState {
    fn new(capacity: usize, page_size: u32, block_size: u32) -> Self {
        Self {
            memory: vec![0xFF; capacity],
            page_buffer: vec![0xFF; page_size as usize],
            status_register: 0,
            config_register: 0,
            write_enabled: false,
            current_row_addr: 0,
            column_ptr: 0,
            page_size,
            block_size,
        }
    }
}

/// A programmer implementation that simulates a connected SPI Flash chip
pub struct SimulatedProgrammer {
    /// Chip State
    state: RefCell<SpiNandState>,
    /// Currently receiving command?
    current_command: RefCell<Option<u8>>,
    /// Buffer for collecting address/dummy bytes after a command
    cmd_buffer: RefCell<Vec<u8>>,
}

impl SimulatedProgrammer {
    pub fn new(capacity: usize, page_size: u32, block_size: u32) -> Self {
        Self {
            state: RefCell::new(SpiNandState::new(capacity, page_size, block_size)),
            current_command: RefCell::new(None),
            cmd_buffer: RefCell::new(Vec::new()),
        }
    }

    /// Get a reference to the internal memory for verification
    pub fn get_memory(&self) -> Vec<u8> {
        self.state.borrow().memory.clone()
    }

    /// Initialize memory with data
    pub fn set_memory(&self, data: &[u8]) {
        let mut state = self.state.borrow_mut();
        let len = std::cmp::min(state.memory.len(), data.len());
        state.memory[..len].copy_from_slice(&data[..len]);
    }

    fn handle_spi_byte(&self, byte: u8) -> u8 {
        let mut state_ref = self.state.borrow_mut();
        let state = &mut *state_ref;
        let mut cmd = self.current_command.borrow_mut();
        let mut buf = self.cmd_buffer.borrow_mut();

        if cmd.is_none() {
            // New command start
            *cmd = Some(byte);
            buf.clear();

            // Handle single-byte commands immediately
            if byte == 0x06 {
                state.write_enabled = true;
                return 0xFF;
            } else if byte == 0x04 {
                state.write_enabled = false;
                return 0xFF;
            }

            return 0xFF; // Hi-Z / Dummy return
        }

        let opcode = cmd.unwrap();
        buf.push(byte); // Store incoming address/dummy/data bytes

        match opcode {
            // READ ID (0x9F)
            0x9F => {
                match buf.len() {
                    1 => 0xEF, // Dummy Manufacturer (Winbond-ish)
                    2 => 0xAA, // Dummy Device
                    3 => 0x21, // Dummy Density
                    _ => 0xFF,
                }
            }
            // GET FEATURE (0x0F)
            0x0F => {
                if buf.len() == 2 {
                    // Address (buf[0]) received in previous cycle
                    // Now receiving dummy/clock for data, return the value
                    let addr = buf[0];
                    match addr {
                        0xC0 => state.status_register, // Status
                        _ => 0x00,
                    }
                } else {
                    0xFF
                }
            }
            // SET FEATURE (0x1F)
            0x1F => {
                if buf.len() == 2 {
                    let addr = buf[0];
                    let val = buf[1];
                    if addr == 0xC0 {
                        state.status_register = val;
                    }
                }
                0xFF
            }
            // WRITE ENABLE/DISABLE managed above
            0x06 | 0x04 => 0xFF,

            // PAGE READ (0x13)
            0x13 => {
                if buf.len() == 3 {
                    let row_addr =
                        ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);
                    state.current_row_addr = row_addr;

                    // Simulate load from array to cache
                    // Calculate flat address
                    let flat_addr = row_addr * state.page_size;
                    let end_addr = flat_addr + state.page_size;

                    if (end_addr as usize) <= state.memory.len() {
                        state
                            .page_buffer
                            .copy_from_slice(&state.memory[flat_addr as usize..end_addr as usize]);
                    }

                    // Not busy (simulated instant)
                }
                0xFF
            }
            // READ CACHE (0x03)
            0x03 => {
                // buf[0]=ColH, buf[1]=ColL, buf[2]=Dummy
                if buf.len() == 3 {
                    // Setup column pointer
                    let col_addr = ((buf[0] as u16) << 8) | (buf[1] as u16);
                    state.column_ptr = col_addr;
                    0xFF // Dummy byte return
                } else if buf.len() > 3 {
                    // Return data
                    let ptr = state.column_ptr as usize;
                    if ptr < state.page_buffer.len() {
                        let data = state.page_buffer[ptr];
                        state.column_ptr += 1;
                        data
                    } else {
                        0xFF
                    }
                } else {
                    0xFF
                }
            }
            // PROGRAM LOAD (0x02)
            0x02 => {
                if buf.len() == 2 {
                    // Col Addr set
                    let col_addr = ((buf[0] as u16) << 8) | (buf[1] as u16);
                    state.column_ptr = col_addr;
                } else if buf.len() > 2 {
                    // Data incoming
                    let ptr = state.column_ptr as usize;
                    if ptr < state.page_buffer.len() {
                        state.page_buffer[ptr] = byte;
                        state.column_ptr += 1;
                    }
                }
                0xFF
            }
            // PROGRAM EXECUTE (0x10)
            0x10 => {
                if buf.len() == 3 && state.write_enabled {
                    let row_addr =
                        ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);

                    let flat_addr = row_addr * state.page_size;
                    let end_addr = flat_addr + state.page_size;

                    if (end_addr as usize) <= state.memory.len() {
                        state.memory[flat_addr as usize..end_addr as usize]
                            .copy_from_slice(&state.page_buffer);
                    }

                    // Clear WEL
                    state.write_enabled = false;
                }
                0xFF
            }
            // BLOCK ERASE (0xD8)
            0xD8 => {
                if buf.len() == 3 && state.write_enabled {
                    let row_addr =
                        ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);
                    // Block erase ignores lower bits of row addr usually
                    let pages_per_block = state.block_size / state.page_size;
                    let block_start_page = (row_addr / pages_per_block) * pages_per_block;

                    let start_addr = block_start_page * state.page_size;
                    let end_addr = start_addr + state.block_size;

                    if (end_addr as usize) <= state.memory.len() {
                        for i in start_addr as usize..end_addr as usize {
                            state.memory[i] = 0xFF; // Erase to 0xFF
                        }
                    }
                    state.write_enabled = false;
                }
                0xFF
            }
            _ => 0xFF,
        }
    }
}

impl Programmer for SimulatedProgrammer {
    fn name(&self) -> &str {
        "SimulatedProgrammer"
    }

    fn spi_transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<()> {
        for (i, &byte) in tx.iter().enumerate() {
            let ret = self.handle_spi_byte(byte);
            if i < rx.len() {
                rx[i] = ret;
            }
        }
        Ok(())
    }

    fn set_cs(&mut self, active: bool) -> Result<()> {
        if !active {
            // CS rising edge: Reset command state
            *self.current_command.borrow_mut() = None;
            self.cmd_buffer.borrow_mut().clear();
        }
        Ok(())
    }

    fn spi_read_bulk(&mut self, len: usize) -> Result<Vec<u8>> {
        let mut rx = vec![0u8; len];
        // In bulk read, we just send dummy bytes (0x00) and read responses
        let tx = vec![0u8; len];
        self.spi_transfer(&tx, &mut rx)?;
        Ok(rx)
    }

    fn max_bulk_transfer_size(&self) -> usize {
        1024 * 1024 // Unlimited for sim
    }
}
