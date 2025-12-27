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
        self.chips.iter().find(|c| c.jedec_id == id).cloned()
    }

    pub fn list_all(&self) -> Vec<ChipSpec> {
        self.chips.clone()
    }
}
