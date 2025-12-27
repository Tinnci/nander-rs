//! SPI Flash command definitions
//!
//! This module defines the standard JEDEC SPI Flash command set
//! used by both NAND and NOR flash chips.

// ============================================================================
// Common Commands (NAND & NOR)
// ============================================================================

/// Read JEDEC ID (Manufacturer ID, Device ID)
pub const CMD_JEDEC_ID: u8 = 0x9F;

/// Read status register
pub const CMD_READ_STATUS: u8 = 0x0F;

/// Read status register (alternative, some NOR chips)
pub const CMD_READ_STATUS_ALT: u8 = 0x05;

/// Write enable
pub const CMD_WRITE_ENABLE: u8 = 0x06;

/// Write disable
pub const CMD_WRITE_DISABLE: u8 = 0x04;

/// Reset device
pub const CMD_RESET: u8 = 0xFF;

// ============================================================================
// SPI NOR Commands
// ============================================================================

/// Read data (standard, up to ~50MHz)
pub const CMD_NOR_READ: u8 = 0x03;

/// Fast read (requires dummy byte)
pub const CMD_NOR_FAST_READ: u8 = 0x0B;

/// Page program (write)
pub const CMD_NOR_PAGE_PROGRAM: u8 = 0x02;

/// Sector erase (4KB)
pub const CMD_NOR_SECTOR_ERASE_4K: u8 = 0x20;

/// Block erase (32KB)
pub const CMD_NOR_BLOCK_ERASE_32K: u8 = 0x52;

/// Block erase (64KB)
pub const CMD_NOR_BLOCK_ERASE_64K: u8 = 0xD8;

/// Chip erase
pub const CMD_NOR_CHIP_ERASE: u8 = 0xC7;

/// Chip erase (alternative)
pub const CMD_NOR_CHIP_ERASE_ALT: u8 = 0x60;

// ============================================================================
// SPI NOR 4-Byte Address Mode Commands (for >16MB chips)
// ============================================================================

/// Enter 4-byte address mode
pub const CMD_NOR_ENTER_4BYTE_MODE: u8 = 0xB7;

/// Exit 4-byte address mode
pub const CMD_NOR_EXIT_4BYTE_MODE: u8 = 0xE9;

/// Read data with 4-byte address
pub const CMD_NOR_READ_4B: u8 = 0x13;

/// Fast read with 4-byte address (requires dummy byte)
pub const CMD_NOR_FAST_READ_4B: u8 = 0x0C;

/// Page program with 4-byte address
pub const CMD_NOR_PAGE_PROGRAM_4B: u8 = 0x12;

/// Sector erase (4KB) with 4-byte address
pub const CMD_NOR_SECTOR_ERASE_4K_4B: u8 = 0x21;

/// Block erase (64KB) with 4-byte address
pub const CMD_NOR_BLOCK_ERASE_64K_4B: u8 = 0xDC;

// ============================================================================
// SPI NAND Commands
// ============================================================================

/// Page read to cache
pub const CMD_NAND_PAGE_READ: u8 = 0x13;

/// Read from cache
pub const CMD_NAND_READ_CACHE: u8 = 0x03;

/// Read from cache (x1, with column address)
pub const CMD_NAND_READ_CACHE_X1: u8 = 0x0B;

/// Read from cache (x2, dual output)
pub const CMD_NAND_READ_CACHE_X2: u8 = 0x3B;

/// Read from cache (x4, quad output)
pub const CMD_NAND_READ_CACHE_X4: u8 = 0x6B;

/// Program load (write to cache)
pub const CMD_NAND_PROGRAM_LOAD: u8 = 0x02;

/// Program load (random data input)
pub const CMD_NAND_PROGRAM_LOAD_RANDOM: u8 = 0x84;

/// Program execute (write cache to array)
pub const CMD_NAND_PROGRAM_EXECUTE: u8 = 0x10;

/// Block erase
pub const CMD_NAND_BLOCK_ERASE: u8 = 0xD8;

/// Get feature (read status register)
pub const CMD_NAND_GET_FEATURE: u8 = 0x0F;

/// Set feature (write configuration)
pub const CMD_NAND_SET_FEATURE: u8 = 0x1F;

// ============================================================================
// Status Register Bits
// ============================================================================

/// SPI NAND Status Register - Operation In Progress (OIP)
pub const STATUS_NAND_OIP: u8 = 0x01;

/// SPI NAND Status Register - Write Enable Latch (WEL)
pub const STATUS_NAND_WEL: u8 = 0x02;

/// SPI NAND Status Register - Erase Fail (E_FAIL)
pub const STATUS_NAND_E_FAIL: u8 = 0x04;

/// SPI NAND Status Register - Program Fail (P_FAIL)
pub const STATUS_NAND_P_FAIL: u8 = 0x08;

/// SPI NAND Status Register - ECC Status bits (ECCS0, ECCS1)
/// 00 = No errors
/// 01 = 1-4 bit errors corrected
/// 10 = More than 4 bits corrected (some chips report as uncorrectable)
/// 11 = Uncorrectable errors
pub const STATUS_NAND_ECC_MASK: u8 = 0x30;

/// ECC Status: No errors detected
pub const STATUS_NAND_ECC_OK: u8 = 0x00;

/// ECC Status: Errors corrected
pub const STATUS_NAND_ECC_CORRECTED: u8 = 0x10;

/// ECC Status: Errors corrected (alternate, some chips)
pub const STATUS_NAND_ECC_CORRECTED_ALT: u8 = 0x20;

/// ECC Status: Uncorrectable errors
pub const STATUS_NAND_ECC_UNCORRECTABLE: u8 = 0x30;

/// SPI NOR Status Register - Write In Progress (WIP)
pub const STATUS_NOR_WIP: u8 = 0x01;

/// SPI NOR Status Register - Write Enable Latch (WEL)
pub const STATUS_NOR_WEL: u8 = 0x02;

// ============================================================================
// Feature Register Addresses (NAND)
// ============================================================================

/// Protection register
pub const FEATURE_PROTECTION: u8 = 0xA0;

/// Feature/Configuration register
pub const FEATURE_CONFIG: u8 = 0xB0;

/// Status register
pub const FEATURE_STATUS: u8 = 0xC0;

/// Drive strength register
pub const FEATURE_DRIVE_STRENGTH: u8 = 0xD0;

// ============================================================================
// SPI EEPROM Commands (25xxx series)
// ============================================================================

/// SPI EEPROM Read data (standard SPI read)
/// Same as NOR Flash: CMD + Address + Data
pub const CMD_EEPROM_READ: u8 = 0x03;

/// SPI EEPROM Write data (byte or page program)
/// Same as NOR Flash: CMD + Address + Data
pub const CMD_EEPROM_WRITE: u8 = 0x02;

/// SPI EEPROM Write Enable
pub const CMD_EEPROM_WREN: u8 = 0x06;

/// SPI EEPROM Write Disable
pub const CMD_EEPROM_WRDI: u8 = 0x04;

/// SPI EEPROM Read Status Register
pub const CMD_EEPROM_RDSR: u8 = 0x05;

/// SPI EEPROM Write Status Register
pub const CMD_EEPROM_WRSR: u8 = 0x01;

// ============================================================================
// SPI EEPROM Status Register Bits
// ============================================================================

/// SPI EEPROM Status Register - Write In Progress (WIP)
pub const STATUS_EEPROM_WIP: u8 = 0x01;

/// SPI EEPROM Status Register - Write Enable Latch (WEL)
pub const STATUS_EEPROM_WEL: u8 = 0x02;

/// SPI EEPROM Status Register - Block Protect bit 0
pub const STATUS_EEPROM_BP0: u8 = 0x04;

/// SPI EEPROM Status Register - Block Protect bit 1
pub const STATUS_EEPROM_BP1: u8 = 0x08;

// ============================================================================
// I2C EEPROM Constants (24Cxx series)
// ============================================================================

/// Default I2C base address for 24Cxx EEPROMs (A0, A1, A2 pins low)
pub const I2C_ADDR_24CXX: u8 = 0xA0;

// ============================================================================
// Microwire EEPROM Opcodes (93Cxx series)
// ============================================================================

/// Microwire READ opcode (3-bit: 110b)
pub const MW_OP_READ: u8 = 0b110;

/// Microwire WRITE opcode (3-bit: 101b)
pub const MW_OP_WRITE: u8 = 0b101;

/// Microwire ERASE opcode (3-bit: 111b)
pub const MW_OP_ERASE: u8 = 0b111;

/// Microwire EWEN (Erase/Write Enable) opcode (3-bit: 100b, addr: 11xxxx)
pub const MW_OP_EWEN: u8 = 0b100;

/// Microwire EWDS (Erase/Write Disable) opcode (3-bit: 100b, addr: 00xxxx)
pub const MW_OP_EWDS: u8 = 0b100;

/// Microwire ERAL (Erase All) opcode (3-bit: 100b, addr: 10xxxx)
pub const MW_OP_ERAL: u8 = 0b100;

/// Microwire WRAL (Write All) opcode (3-bit: 100b, addr: 01xxxx)
pub const MW_OP_WRAL: u8 = 0b100;

// ============================================================================
// Configuration Register Bits (NAND - address 0xB0)
// ============================================================================

/// ECC Enable bit in configuration register
pub const CONFIG_ECC_ENABLE: u8 = 0x10;

/// OTP Enable bit
pub const CONFIG_OTP_ENABLE: u8 = 0x40;

/// OTP Protect bit
pub const CONFIG_OTP_PROTECT: u8 = 0x80;

/// Buffer mode bit (some chips)
pub const CONFIG_BUF_MODE: u8 = 0x08;
