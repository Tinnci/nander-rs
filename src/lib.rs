//! nander-rs library
//!
//! This library provides functionality for programming SPI NAND/NOR Flash chips
//! using hardware programmers like CH341A.
//!
//! # Architecture
//!
//! The library is organized into layered architecture:
//!
//! - [`domain`]: Core business logic and types
//! - [`application`]: Use cases and business rules
//! - [`infrastructure`]: Technology-specific implementations (Programmers, Database)
//! - [`presentation`]: User interfaces (CLI)
//! - [`error`]: Error types and handling
//!
//! # Example
//!
//! use nander_rs::presentation::cli::args::Args;
//! use nander_rs::presentation::cli;
//! use clap::Parser;
//!
//! // Parse and execute
//! let args = Args::parse();
//! if let Err(e) = cli::execute(args) {
//!     eprintln!("Error: {}", e);
//! }
//! ```

// --- New Layered Architecture ---
pub mod application;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod presentation;

pub use error::{Error, Result};
