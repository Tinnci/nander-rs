//! EEPROM Protocol Implementations
//!
//! This module contains protocol implementations for various EEPROM types:
//!
//! - `spi_25xxx` - SPI EEPROM (25xxx series)
//! - `i2c_24cxx` - I2C EEPROM (24Cxx series) [planned]
//! - `microwire_93cxx` - Microwire EEPROM (93Cxx series) [planned]

pub mod i2c_24cxx;
pub mod microwire_93cxx;
pub mod spi_25xxx;

pub use i2c_24cxx::I2cEeprom;
pub use microwire_93cxx::MicrowireEeprom;
pub use spi_25xxx::SpiEeprom;
