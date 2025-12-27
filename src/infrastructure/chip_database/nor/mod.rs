//! NOR Chip Database - Manufacturer Modules
//!
//! This module provides access to all supported SPI NOR flash chips,
//! organized by manufacturer.
//!
//! # Supported Manufacturers
//!
//! - GigaDevice (0xC8) - 5 chips
//! - Winbond (0xEF) - 23 chips
//! - Macronix (0xC2) - 8 chips
//! - Spansion (0x01) - 11 chips
//! - Micron/ST (0x20) - 13 chips
//! - EON (0x1C) - 14 chips
//! - Others (Atmel, ESMT, Zbit, BOYA, XTX, ISSI, FM, Zetta, PUYA) - 54 chips
//!
//! Total: ~128 NOR chips

pub mod eon;
pub mod fram;
pub mod gigadevice;
pub mod macronix;
pub mod micron;
pub mod others;
pub mod spansion;
pub mod winbond;

use crate::domain::ChipSpec;

/// Returns all NOR chips from all manufacturers
pub fn get_all_nor() -> Vec<ChipSpec> {
    let mut chips = Vec::new();

    // Major manufacturers
    chips.extend(gigadevice::get_chips());
    chips.extend(winbond::get_chips());
    chips.extend(macronix::get_chips());
    chips.extend(spansion::get_chips());
    chips.extend(micron::get_chips());
    chips.extend(eon::get_chips());

    // Other smaller manufacturers
    chips.extend(others::get_chips());
    chips.extend(fram::get_chips());

    chips
}
