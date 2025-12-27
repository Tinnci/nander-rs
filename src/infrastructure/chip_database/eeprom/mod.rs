//! EEPROM Chip Database
//!
//! This module contains chip definitions for various EEPROM types:
//!
//! - `spi_25xxx` - SPI EEPROM (25xxx series)
pub mod i2c_24cxx;
pub mod microwire_93cxx;
pub mod spi_25xxx;

use crate::domain::ChipSpec;

/// Returns all EEPROM chips from all types
pub fn get_all_eeprom() -> Vec<ChipSpec> {
    let mut chips = Vec::new();
    chips.extend(spi_25xxx::get_all_spi_eeprom());
    chips.extend(i2c_24cxx::get_all_i2c_eeprom());
    chips.extend(microwire_93cxx::get_all_microwire_eeprom());
    chips
}
