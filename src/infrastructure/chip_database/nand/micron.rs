//! Micron NAND chips

use crate::domain::chip::*;
use crate::domain::types::*;

pub fn get_chips() -> Vec<ChipSpec> {
    vec![
        // MT29F1G01ABAFD
        ChipSpec {
            name: "MT29F1G01ABAFD".to_string(),
            manufacturer: "Micron".to_string(),
            jedec_id: JedecId::new([0x2C, 0x14, 0x00]),
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
        // MT29F2G01ABAGD
        ChipSpec {
            name: "MT29F2G01ABAGD".to_string(),
            manufacturer: "Micron".to_string(),
            jedec_id: JedecId::new([0x2C, 0x24, 0x00]),
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
        // Add more Micron chips here...
    ]
}
