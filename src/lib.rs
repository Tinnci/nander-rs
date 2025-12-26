//! nander-rs library
//!
//! This library provides functionality for programming SPI NAND/NOR Flash chips
//! using hardware programmers like CH341A.
//!
//! # Architecture
//!
//! The library is organized into several modules:
//!
//! - [`hardware`]: Hardware abstraction layer for different programmers
//! - [`flash`]: Flash chip protocol implementations (NAND/NOR)
//! - [`database`]: Chip database with specifications for supported Flash chips
//! - [`error`]: Error types and handling
//!
//! # Example
//!
//! ```no_run
//! use nander_rs::hardware::ch341a::Ch341a;
//! use nander_rs::hardware::Programmer;
//! use nander_rs::flash::FlashOps;
//!
//! // Discover and open programmer
//! let mut programmer = Ch341a::open()?;
//!
//! // Detect flash chip
//! let chip_id = programmer.read_jedec_id()?;
//! println!("Detected chip: {:02X?}", chip_id);
//! ```

// --- New Layered Architecture ---
pub mod application;
pub mod domain;
pub mod infrastructure;

// --- Legacy Modules (To be phased out) ---
pub mod cli;
pub mod database;
pub mod error;
pub mod flash;
pub mod hardware;

pub use error::{Error, Result};
