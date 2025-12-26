//! Infrastructure - Flash Protocols
//!
//! Implementation of SPI NAND and SPI NOR Flash protocols.

pub mod commands;
pub mod nand;
pub mod nor;

use super::programmer::Programmer;
use crate::domain::FlashOperation;

/// A protocol handler that bridges a Programmer with FlashOperations
#[allow(dead_code)]
pub struct ProtocolHandler<P: Programmer> {
    programmer: P,
    implementation: Box<dyn FlashOperation>,
}

// Note: In a real implementation, we would have specific builders
// for NAND and NOR that wrap the programmer.
