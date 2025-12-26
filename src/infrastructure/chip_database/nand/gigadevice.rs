//! GigaDevice NAND chips

use crate::domain::chip::*;
use crate::domain::types::*;

pub fn get_chips() -> Vec<ChipSpec> {
    vec![
        // GD5F1GM7UE
        ChipSpec {
            name: "GD5F1GM7UE".to_string(),
            manufacturer: "GigaDevice".to_string(),
            jedec_id: JedecId::new([0xC8, 0x91, 0x00]),
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
        // Add more GigaDevice chips here...
    ]
}
