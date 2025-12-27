//! NAND Chip Database - Manufacturer Modules
//!
//! This module provides access to all supported SPI NAND flash chips,
//! organized by manufacturer.
//!
//! # Supported Manufacturers
//!
//! - GigaDevice (0xC8) - 12 chips
//! - Winbond (0xEF) - 4 chips
//! - Micron (0x2C) - 3 chips
//! - Macronix (0xC2) - 4 chips
//! - Toshiba/Kioxia (0x98) - 4 chips
//! - ESMT (0xC8) - 5 chips
//! - Etron (0xD5) - 15 chips
//! - XTX (0x0B) - 4 chips
//! - FORESEE (0xCD) - 5 chips
//! - Others (HEYANG, PN, ATO, FM, DS, Zentel) - 23 chips
//!
//! Total: ~79 NAND chips

pub mod esmt;
pub mod etron;
pub mod foresee;
pub mod gigadevice;
pub mod macronix;
pub mod micron;
pub mod others;
pub mod toshiba;
pub mod winbond;
pub mod xtx;

use crate::domain::ChipSpec;

/// Returns all NAND chips from all manufacturers
pub fn get_all_nand() -> Vec<ChipSpec> {
    let mut chips = Vec::new();

    // Major manufacturers
    chips.extend(gigadevice::get_chips());
    chips.extend(winbond::get_chips());
    chips.extend(micron::get_chips());
    chips.extend(macronix::get_chips());
    chips.extend(toshiba::get_chips());

    // Secondary manufacturers
    chips.extend(esmt::get_chips());
    chips.extend(etron::get_chips());
    chips.extend(xtx::get_chips());
    chips.extend(foresee::get_chips());

    // Other smaller manufacturers
    chips.extend(others::get_chips());

    chips
}
