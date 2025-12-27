//! SPI EEPROM (25xxx Series) Chip Database
//!
//! This module defines supported SPI EEPROM chips.
//!
//! # Supported Chips
//!
//! The 25xxx series SPI EEPROMs are similar to SPI NOR Flash but:
//! - Much smaller capacity (128 bytes to 128KB)
//! - Smaller page sizes (typically 16-256 bytes)
//! - No sector/block erase required (byte-writable)
//! - Usually don't have JEDEC ID (use manual selection)
//!
//! # Manufacturers
//!
//! - Atmel (AT25xxx)
//! - Microchip (25xxx, 25LCxxx)
//! - STMicroelectronics (M95xxx)
//! - ISSI (IS25Cxx)
//! - Catalyst (CAT25xxx)

use crate::domain::chip::*;
use crate::domain::types::*;

/// Get all SPI EEPROM chip definitions
pub fn get_all_spi_eeprom() -> Vec<ChipSpec> {
    let mut chips = Vec::new();
    chips.extend(atmel_chips());
    chips.extend(microchip_chips());
    chips.extend(st_chips());
    chips
}

// ============================================================================
// Atmel AT25xxx Series
// ============================================================================

fn atmel_chips() -> Vec<ChipSpec> {
    vec![
        // AT25xxx - Standard SPI EEPROM
        spi_eeprom("AT25010", "Atmel", 128, 8), // 1Kbit = 128 bytes
        spi_eeprom("AT25020", "Atmel", 256, 8), // 2Kbit = 256 bytes
        spi_eeprom("AT25040", "Atmel", 512, 8), // 4Kbit = 512 bytes
        spi_eeprom("AT25080", "Atmel", 1024, 32), // 8Kbit = 1KB
        spi_eeprom("AT25160", "Atmel", 2048, 32), // 16Kbit = 2KB
        spi_eeprom("AT25320", "Atmel", 4096, 32), // 32Kbit = 4KB
        spi_eeprom("AT25640", "Atmel", 8192, 32), // 64Kbit = 8KB
        spi_eeprom("AT25128", "Atmel", 16384, 64), // 128Kbit = 16KB
        spi_eeprom("AT25256", "Atmel", 32768, 64), // 256Kbit = 32KB
        spi_eeprom("AT25512", "Atmel", 65536, 128), // 512Kbit = 64KB
        spi_eeprom("AT25M01", "Atmel", 131072, 256), // 1Mbit = 128KB
    ]
}

// ============================================================================
// Microchip 25xxx / 25LCxxx Series
// ============================================================================

fn microchip_chips() -> Vec<ChipSpec> {
    vec![
        // 25LCxxx - Standard 3.3V SPI EEPROM
        spi_eeprom("25LC010A", "Microchip", 128, 16),
        spi_eeprom("25LC020A", "Microchip", 256, 16),
        spi_eeprom("25LC040A", "Microchip", 512, 16),
        spi_eeprom("25LC080A", "Microchip", 1024, 16),
        spi_eeprom("25LC160A", "Microchip", 2048, 16),
        spi_eeprom("25LC320A", "Microchip", 4096, 32),
        spi_eeprom("25LC640A", "Microchip", 8192, 32),
        spi_eeprom("25LC128", "Microchip", 16384, 64),
        spi_eeprom("25LC256", "Microchip", 32768, 64),
        spi_eeprom("25LC512", "Microchip", 65536, 128),
        spi_eeprom("25LC1024", "Microchip", 131072, 256),
    ]
}

// ============================================================================
// STMicroelectronics M95xxx Series
// ============================================================================

fn st_chips() -> Vec<ChipSpec> {
    vec![
        // M95xxx - Automotive grade SPI EEPROM
        spi_eeprom("M95010", "STMicroelectronics", 128, 16),
        spi_eeprom("M95020", "STMicroelectronics", 256, 16),
        spi_eeprom("M95040", "STMicroelectronics", 512, 16),
        spi_eeprom("M95080", "STMicroelectronics", 1024, 32),
        spi_eeprom("M95160", "STMicroelectronics", 2048, 32),
        spi_eeprom("M95320", "STMicroelectronics", 4096, 32),
        spi_eeprom("M95640", "STMicroelectronics", 8192, 32),
        spi_eeprom("M95128", "STMicroelectronics", 16384, 64),
        spi_eeprom("M95256", "STMicroelectronics", 32768, 64),
        spi_eeprom("M95512", "STMicroelectronics", 65536, 128),
        spi_eeprom("M95M01", "STMicroelectronics", 131072, 256),
    ]
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Helper function to create a SPI EEPROM chip spec
///
/// Note: Most SPI EEPROMs don't have a standard JEDEC ID like NOR Flash.
/// We use synthetic IDs based on capacity for identification:
/// - Manufacturer: 0xFE (synthetic marker for SPI EEPROM)
/// - Device: capacity indicator
/// - Density: page size indicator
fn spi_eeprom(name: &str, manufacturer: &str, capacity_bytes: u32, page_size: u32) -> ChipSpec {
    // Generate synthetic JEDEC ID for SPI EEPROM
    // Format: 0xFE (SPI EEPROM marker) | capacity_code | page_code
    let capacity_code = match capacity_bytes {
        128 => 0x01,
        256 => 0x02,
        512 => 0x03,
        1024 => 0x04,
        2048 => 0x05,
        4096 => 0x06,
        8192 => 0x07,
        16384 => 0x08,
        32768 => 0x09,
        65536 => 0x0A,
        131072 => 0x0B,
        _ => 0x00,
    };

    let page_code = match page_size {
        8 => 0x01,
        16 => 0x02,
        32 => 0x03,
        64 => 0x04,
        128 => 0x05,
        256 => 0x06,
        _ => 0x00,
    };

    ChipSpec {
        name: name.to_string(),
        manufacturer: manufacturer.to_string(),
        jedec_id: JedecId::new([0xFE, capacity_code, page_code]),
        flash_type: FlashType::SpiEeprom,
        capacity: Capacity::bytes(capacity_bytes),
        layout: ChipLayout {
            page_size,
            block_size: page_size, // EEPROMs are byte-writable, use page as block
            oob_size: None,
        },
        capabilities: ChipCapabilities {
            supports_4byte_addr: capacity_bytes > 65536,
            ..Default::default()
        },
    }
}
