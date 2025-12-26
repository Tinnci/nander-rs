//! GigaDevice NOR chips

use crate::domain::chip::*;
use crate::domain::types::*;

pub fn get_chips() -> Vec<ChipSpec> {
    vec![
        // GD25Q128C
        ChipSpec {
            name: "GD25Q128C".to_string(),
            manufacturer: "GigaDevice".to_string(),
            jedec_id: JedecId::new([0xC8, 0x40, 0x18]),
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
        // GD25Q64C
        ChipSpec {
            name: "GD25Q64C".to_string(),
            manufacturer: "GigaDevice".to_string(),
            jedec_id: JedecId::new([0xC8, 0x40, 0x17]),
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
        // Add more GigaDevice NOR chips here...
    ]
}
