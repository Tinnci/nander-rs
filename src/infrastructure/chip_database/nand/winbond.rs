//! Winbond NAND chips

use crate::domain::chip::*;
use crate::domain::types::*;

pub fn get_chips() -> Vec<ChipSpec> {
    vec![
        // W25N01GVZEIG
        ChipSpec {
            name: "W25N01GVZEIG".to_string(),
            manufacturer: "Winbond".to_string(),
            jedec_id: JedecId::new([0xEF, 0xAA, 0x21]),
            flash_type: FlashType::Nand,
            capacity: Capacity::gigabits(1),
            layout: ChipLayout {
                page_size: 2048,
                block_size: 128 * 1024,
                oob_size: Some(64),
            },
            capabilities: ChipCapabilities {
                supports_ecc_control: true,
                ..Default::default()
            },
        },
        // W25N02KVZEIR
        ChipSpec {
            name: "W25N02KVZEIR".to_string(),
            manufacturer: "Winbond".to_string(),
            jedec_id: JedecId::new([0xEF, 0xAA, 0x22]),
            flash_type: FlashType::Nand,
            capacity: Capacity::gigabits(2),
            layout: ChipLayout {
                page_size: 2048,
                block_size: 128 * 1024,
                oob_size: Some(64),
            },
            capabilities: ChipCapabilities {
                supports_ecc_control: true,
                ..Default::default()
            },
        },
        // Add more Winbond chips here...
    ]
}
