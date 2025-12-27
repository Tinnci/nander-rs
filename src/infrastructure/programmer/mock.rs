//! Mock Programmer for testing
//!
//! This module provides a mock implementation of the Programmer trait
//! for unit testing flash protocol implementations without actual hardware.

use crate::error::Result;
use crate::infrastructure::programmer::Programmer;
use std::cell::RefCell;
use std::collections::VecDeque;

/// A mock programmer that records all operations and returns pre-configured responses
#[derive(Debug)]
pub struct MockProgrammer {
    /// Name of this mock programmer
    name: String,
    /// Current CS state
    cs_active: RefCell<bool>,
    /// Recorded SPI write data
    write_log: RefCell<Vec<Vec<u8>>>,
    /// Pre-configured read responses (FIFO queue)
    read_responses: RefCell<VecDeque<Vec<u8>>>,
    /// Transaction log for debugging
    transaction_log: RefCell<Vec<Transaction>>,
}

/// Record of a single SPI transaction
#[derive(Debug, Clone)]
pub enum Transaction {
    CsActive(bool),
    Write(Vec<u8>),
    Read { len: usize, data: Vec<u8> },
    Transfer { tx: Vec<u8>, rx: Vec<u8> },
}

impl MockProgrammer {
    /// Create a new mock programmer
    pub fn new() -> Self {
        Self {
            name: "MockProgrammer".to_string(),
            cs_active: RefCell::new(false),
            write_log: RefCell::new(Vec::new()),
            read_responses: RefCell::new(VecDeque::new()),
            transaction_log: RefCell::new(Vec::new()),
        }
    }

    /// Set the name of this mock programmer
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// Add a read response to the queue
    /// Responses are returned in FIFO order
    pub fn expect_read(&self, data: Vec<u8>) {
        self.read_responses.borrow_mut().push_back(data);
    }

    /// Add multiple read responses
    pub fn expect_reads(&self, responses: Vec<Vec<u8>>) {
        for data in responses {
            self.expect_read(data);
        }
    }

    /// Get all recorded write operations
    pub fn get_writes(&self) -> Vec<Vec<u8>> {
        self.write_log.borrow().clone()
    }

    /// Get the transaction log
    pub fn get_transactions(&self) -> Vec<Transaction> {
        self.transaction_log.borrow().clone()
    }

    /// Check if CS is currently active
    pub fn is_cs_active(&self) -> bool {
        *self.cs_active.borrow()
    }

    /// Clear all logs and responses
    pub fn reset(&self) {
        self.write_log.borrow_mut().clear();
        self.read_responses.borrow_mut().clear();
        self.transaction_log.borrow_mut().clear();
        *self.cs_active.borrow_mut() = false;
    }
}

impl Default for MockProgrammer {
    fn default() -> Self {
        Self::new()
    }
}

impl Programmer for MockProgrammer {
    fn name(&self) -> &str {
        &self.name
    }

    fn spi_transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<()> {
        // Log the write
        self.write_log.borrow_mut().push(tx.to_vec());

        // Get pre-configured response or return 0xFF
        let response = self
            .read_responses
            .borrow_mut()
            .pop_front()
            .unwrap_or_else(|| vec![0xFF; rx.len()]);

        // Copy response to rx buffer
        let copy_len = rx.len().min(response.len());
        rx[..copy_len].copy_from_slice(&response[..copy_len]);

        // Log transaction
        self.transaction_log
            .borrow_mut()
            .push(Transaction::Transfer {
                tx: tx.to_vec(),
                rx: rx.to_vec(),
            });

        Ok(())
    }

    fn set_cs(&mut self, active: bool) -> Result<()> {
        *self.cs_active.borrow_mut() = active;
        self.transaction_log
            .borrow_mut()
            .push(Transaction::CsActive(active));
        Ok(())
    }

    fn spi_read_bulk(&mut self, len: usize) -> Result<Vec<u8>> {
        let response = self
            .read_responses
            .borrow_mut()
            .pop_front()
            .unwrap_or_else(|| vec![0xFF; len]);

        self.transaction_log.borrow_mut().push(Transaction::Read {
            len,
            data: response.clone(),
        });

        Ok(response)
    }

    fn max_bulk_transfer_size(&self) -> usize {
        4096 // Simulate CH341A bulk size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_programmer_basic() {
        let mut mock = MockProgrammer::new();

        // Test CS control
        assert!(!mock.is_cs_active());
        mock.set_cs(true).unwrap();
        assert!(mock.is_cs_active());
        mock.set_cs(false).unwrap();
        assert!(!mock.is_cs_active());
    }

    #[test]
    fn test_mock_programmer_transfer() {
        let mut mock = MockProgrammer::new();

        // Configure expected read response
        mock.expect_read(vec![0x9F, 0xEF, 0xAA, 0x21]);

        // Do transfer
        let mut rx = [0u8; 4];
        mock.spi_transfer(&[0x9F, 0x00, 0x00, 0x00], &mut rx)
            .unwrap();

        // Verify response
        assert_eq!(rx, [0x9F, 0xEF, 0xAA, 0x21]);

        // Verify write was logged
        let writes = mock.get_writes();
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0], vec![0x9F, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_mock_programmer_multiple_reads() {
        let mut mock = MockProgrammer::new();

        // Configure multiple responses
        mock.expect_reads(vec![vec![0x01], vec![0x02], vec![0x03]]);

        // Read them in order
        let r1 = mock.spi_read(1).unwrap();
        let r2 = mock.spi_read(1).unwrap();
        let r3 = mock.spi_read(1).unwrap();

        assert_eq!(r1, vec![0x01]);
        assert_eq!(r2, vec![0x02]);
        assert_eq!(r3, vec![0x03]);
    }
}
