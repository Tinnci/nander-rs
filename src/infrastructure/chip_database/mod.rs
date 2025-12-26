//! Infrastructure - Chip Database
//!
//! Registry of supported flash chips.

use crate::domain::{ChipSpec, JedecId};

pub mod nand;
pub mod nor;

pub struct ChipRegistry {
    chips: Vec<ChipSpec>,
}

impl ChipRegistry {
    pub fn new() -> Self {
        let mut chips = Vec::new();
        chips.extend(nand::get_all_nand());
        // chips.extend(nor::get_all_nor());
        Self { chips }
    }

    pub fn find_by_id(&self, id: JedecId) -> Option<ChipSpec> {
        self.chips.iter().find(|c| c.jedec_id == id).cloned()
    }

    pub fn list_all(&self) -> Vec<ChipSpec> {
        self.chips.clone()
    }
}
