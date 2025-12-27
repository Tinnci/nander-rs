//! USB Device Database and Recognition System
//!
//! This module maintains a database of known USB devices (especially WCH/QinHeng chips)
//! and provides intelligent device recognition and user guidance.

use std::fmt;

/// USB Device Identification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsbDeviceId {
    pub vendor_id: u16,
    pub product_id: u16,
}

impl UsbDeviceId {
    pub const fn new(vid: u16, pid: u16) -> Self {
        Self {
            vendor_id: vid,
            product_id: pid,
        }
    }
}

/// Device compatibility status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceCompatibility {
    /// Fully supported and ready to use
    Supported,
    /// Device exists but in wrong mode (e.g., UART instead of SPI)
    WrongMode,
    /// Device from same manufacturer but different model
    RelatedDevice,
    /// Planned for future support
    PlannedSupport,
    /// Unknown device
    Unknown,
}

/// Device information entry
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: UsbDeviceId,
    pub name: &'static str,
    pub description: &'static str,
    pub compatibility: DeviceCompatibility,
    pub help_message: Option<&'static str>,
}

impl DeviceInfo {
    /// Get a user-friendly status indicator
    pub fn status_indicator(&self) -> &'static str {
        match self.compatibility {
            DeviceCompatibility::Supported => "✓",
            DeviceCompatibility::WrongMode => "⚠",
            DeviceCompatibility::RelatedDevice => "ℹ",
            DeviceCompatibility::PlannedSupport => "◐",
            DeviceCompatibility::Unknown => "?",
        }
    }

    /// Get colored status for terminal output
    pub fn status_description(&self) -> String {
        match self.compatibility {
            DeviceCompatibility::Supported => "Supported".to_string(),
            DeviceCompatibility::WrongMode => "Wrong Mode".to_string(),
            DeviceCompatibility::RelatedDevice => "Related Device".to_string(),
            DeviceCompatibility::PlannedSupport => "Planned".to_string(),
            DeviceCompatibility::Unknown => "Unknown".to_string(),
        }
    }
}

/// WCH/QinHeng Device Database
pub struct WchDeviceDatabase;

impl WchDeviceDatabase {
    /// VID for WinChipHead / QinHeng Electronics
    pub const WCH_VID: u16 = 0x1A86;

    /// Get device information by VID:PID
    pub fn identify(vid: u16, pid: u16) -> DeviceInfo {
        // Only handle WCH devices
        if vid != Self::WCH_VID {
            return DeviceInfo {
                id: UsbDeviceId::new(vid, pid),
                name: "Unknown Device",
                description: "Not a WCH/QinHeng device",
                compatibility: DeviceCompatibility::Unknown,
                help_message: None,
            };
        }

        // Match against known WCH PIDs
        match pid {
            // ===== CH341 Series =====
            0x5512 => DeviceInfo {
                id: UsbDeviceId::new(vid, pid),
                name: "CH341A",
                description: "USB to SPI/I2C Bridge (Programmer Mode)",
                compatibility: DeviceCompatibility::Supported,
                help_message: Some("This is the correct mode for flash programming!"),
            },

            0x5523 => DeviceInfo {
                id: UsbDeviceId::new(vid, pid),
                name: "CH341",
                description: "USB to Serial Adapter (UART Mode)",
                compatibility: DeviceCompatibility::WrongMode,
                help_message: Some(
                    "⚠ Device is in UART mode, not SPI mode.\n\
                    Solutions:\n\
                    1. Check hardware jumper/switch on your CH341A board (look for SPI/UART or 0/1)\n\
                    2. Some boards require shorting specific pins to enter SPI mode\n\
                    3. Unplug device, change jumper, then replug USB\n\
                    4. After switching, the device should appear as PID 0x5512"
                ),
            },

            0x7523 => DeviceInfo {
                id: UsbDeviceId::new(vid, pid),
                name: "CH340",
                description: "USB to Serial Adapter",
                compatibility: DeviceCompatibility::RelatedDevice,
                help_message: Some(
                    "CH340 is a serial adapter, not a flash programmer.\n\
                    You need a CH341A device in SPI mode (PID 0x5512)."
                ),
            },

            // ===== CH347 Series (High-Speed) =====
            0x55DB => DeviceInfo {
                id: UsbDeviceId::new(vid, pid),
                name: "CH347",
                description: "USB to UART/I2C/SPI Bridge (High-Speed)",
                compatibility: DeviceCompatibility::Supported,
                help_message: Some(
                    "High-speed SPI bridge (up to 60MHz). This is the correct mode for flash programming!"
                ),
            },

            0x55DD => DeviceInfo {
                id: UsbDeviceId::new(vid, pid),
                name: "CH347F",
                description: "USB to JTAG/SWD/UART Bridge",
                compatibility: DeviceCompatibility::RelatedDevice,
                help_message: Some(
                    "CH347F is primarily for JTAG/SWD debugging.\n\
                    For flash programming, use CH341A (0x5512) or CH347 (0x55DB)."
                ),
            },

            // ===== CH348 Series =====
            0x55D2 | 0x55D3 => DeviceInfo {
                id: UsbDeviceId::new(vid, pid),
                name: "CH348",
                description: "USB to Multi-Serial Port",
                compatibility: DeviceCompatibility::RelatedDevice,
                help_message: Some(
                    "CH348 is a multi-port serial adapter, not a flash programmer.\n\
                    You need a CH341A (0x5512) or CH347 (0x55DB) for SPI flash operations."
                ),
            },

            // ===== CH9102/CH9103 Series (Modern Serial) =====
            0x55D4 => DeviceInfo {
                id: UsbDeviceId::new(vid, pid),
                name: "CH9102",
                description: "USB to Serial (Type-C)",
                compatibility: DeviceCompatibility::RelatedDevice,
                help_message: Some("CH9102 is a serial adapter, not a flash programmer."),
            },

            // ===== Unknown WCH Device =====
            _ => DeviceInfo {
                id: UsbDeviceId::new(vid, pid),
                name: "Unknown WCH Device",
                description: "WCH/QinHeng device with unknown PID",
                compatibility: DeviceCompatibility::Unknown,
                help_message: Some(
                    "This is a WCH device, but not recognized by nander-rs.\n\
                    Please report this on our GitHub issues with the device model."
                ),
            },
        }
    }

    /// Check if a device is a supported programmer
    pub fn is_supported_programmer(vid: u16, pid: u16) -> bool {
        let info = Self::identify(vid, pid);
        info.compatibility == DeviceCompatibility::Supported
    }

    /// Get all supported programmer IDs
    pub fn supported_programmers() -> Vec<UsbDeviceId> {
        vec![
            UsbDeviceId::new(Self::WCH_VID, 0x5512), // CH341A SPI Mode
            UsbDeviceId::new(Self::WCH_VID, 0x55DB), // CH347 SPI Mode
        ]
    }
}

impl fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {:04X}:{:04X} - {} ({})",
            self.status_indicator(),
            self.id.vendor_id,
            self.id.product_id,
            self.name,
            self.status_description()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ch341a_spi_mode() {
        let info = WchDeviceDatabase::identify(0x1A86, 0x5512);
        assert_eq!(info.compatibility, DeviceCompatibility::Supported);
        assert_eq!(info.name, "CH341A");
    }

    #[test]
    fn test_ch341_uart_mode() {
        let info = WchDeviceDatabase::identify(0x1A86, 0x5523);
        assert_eq!(info.compatibility, DeviceCompatibility::WrongMode);
        assert!(info.help_message.is_some());
    }

    #[test]
    fn test_ch347_supported() {
        let info = WchDeviceDatabase::identify(0x1A86, 0x55DB);
        assert_eq!(info.compatibility, DeviceCompatibility::Supported);
        assert_eq!(info.name, "CH347");
    }

    #[test]
    fn test_non_wch_device() {
        let info = WchDeviceDatabase::identify(0x0000, 0x0000);
        assert_eq!(info.compatibility, DeviceCompatibility::Unknown);
    }
}
