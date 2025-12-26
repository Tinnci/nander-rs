//! NOR Chip Database - Manufacturer Modules

pub mod gigadevice;
pub mod winbond;
// pub mod macronix;

use crate::domain::ChipSpec;

pub fn get_all_nor() -> Vec<ChipSpec> {
    let mut chips = Vec::new();
    chips.extend(gigadevice::get_chips());
    chips.extend(winbond::get_chips());
    // chips.extend(macronix::get_chips());
    chips
}
