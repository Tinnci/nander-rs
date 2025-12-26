//! NAND Chip Database - Manufacturer Modules

pub mod gigadevice;
pub mod micron;
pub mod winbond;
// etc.

use crate::domain::ChipSpec;

pub fn get_all_nand() -> Vec<ChipSpec> {
    let mut chips = Vec::new();
    chips.extend(gigadevice::get_chips());
    chips.extend(winbond::get_chips());
    chips.extend(micron::get_chips());
    chips
}
