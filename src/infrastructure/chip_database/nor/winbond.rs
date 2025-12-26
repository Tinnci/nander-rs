//! Winbond NOR chips

use crate::domain::chip::*;
use crate::domain::types::*;

pub fn get_chips() -> Vec<ChipSpec> {
    vec![
        // W25Q128JV
        ChipSpec {
            name: "W25Q128JV".to_string(),
            manufacturer: "Winbond".to_string(),
            jedec_id: JedecId::new([0xEF, 0x40, 0x18]),
            flash_type: FlashType::Nor,
            capacity: Capacity::megabytes(16),
            layout: ChipLayout {
                page_size: 256,
                block_size: 64 * 1024,
                oob_size: None,
            },
            capabilities: ChipCapabilities {
                supports_quad_spi: true,
                supports_dual_spi: true,
                ..Default::default()
            },
        },
        // W25Q64JV
        ChipSpec {
            name: "W25Q64JV".to_string(),
            manufacturer: "Winbond".to_string(),
            jedec_id: JedecId::new([0xEF, 0x40, 0x17]),
            flash_type: FlashType::Nor,
            capacity: Capacity::megabytes(8),
            layout: ChipLayout {
                page_size: 256,
                block_size: 64 * 1024,
                oob_size: None,
            },
            capabilities: ChipCapabilities {
                supports_quad_spi: true,
                supports_dual_spi: true,
                ..Default::default()
            },
        },
        // W25Q32JV
        ChipSpec {
            name: "W25Q32JV".to_string(),
            manufacturer: "Winbond".to_string(),
            jedec_id: JedecId::new([0xEF, 0x40, 0x16]),
            flash_type: FlashType::Nor,
            capacity: Capacity::megabytes(4),
            layout: ChipLayout {
                page_size: 256,
                block_size: 64 * 1024,
                oob_size: None,
            },
            capabilities: ChipCapabilities {
                supports_quad_spi: true,
                supports_dual_spi: true,
                ..Default::default()
            },
        },
        // Add more Winbond NOR chips here...
    ]
}
