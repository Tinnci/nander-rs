use super::{eeprom, nand, nor};
use crate::domain::{ChipSpec, JedecId};

#[derive(Clone)]
pub struct ChipRegistry {
    chips: Vec<ChipSpec>,
}

impl Default for ChipRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ChipRegistry {
    pub fn new() -> Self {
        let mut chips = Vec::new();
        chips.extend(nand::get_all_nand());
        chips.extend(nor::get_all_nor());
        chips.extend(eeprom::get_all_eeprom());
        Self { chips }
    }

    pub fn from_specs(chips: Vec<ChipSpec>) -> Self {
        Self { chips }
    }

    pub fn find_by_id(&self, id: JedecId) -> Option<ChipSpec> {
        // 1. Exact match
        if let Some(chip) = self.chips.iter().find(|c| c.jedec_id == id) {
            return Some(chip.clone());
        }

        // 2. SPI NAND match when third byte is shifted due to 1-byte dummy address:
        // When querying with [0xFF, mfr, dev], match NAND chip with [mfr, dev, 0x00]
        if id.manufacturer == 0xFF && id.device != 0xFF {
            if let Some(chip) = self.chips.iter().find(|c| {
                c.flash_type == crate::domain::FlashType::Nand
                    && c.jedec_id.manufacturer == id.device
                    && c.jedec_id.device == id.density
            }) {
                return Some(chip.clone());
            }
        }

        // 3. Fallback for 2-byte NAND IDs where third byte in database is 0x00
        // (e.g. read ID returns [0xC8, 0x41, 0x41] or [0xC8, 0x41, 0x00])
        if id.manufacturer != 0xFF && id.manufacturer != 0x00 {
            if let Some(chip) = self.chips.iter().find(|c| {
                c.flash_type == crate::domain::FlashType::Nand
                    && c.jedec_id.manufacturer == id.manufacturer
                    && c.jedec_id.device == id.device
                    && c.jedec_id.density == 0x00
            }) {
                return Some(chip.clone());
            }
        }

        None
    }

    pub fn list_all(&self) -> Vec<ChipSpec> {
        self.chips.clone()
    }
}
