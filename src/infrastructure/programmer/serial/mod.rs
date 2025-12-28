//! Serial/UART Support Module
//!
//! This module provides serial port functionality for the Console tab,
//! supporting CH340, CH341A UART mode, and CH347 UART mode.

pub mod ch340;
pub mod ch347;

use crate::error::{Error, Result};
use crate::infrastructure::programmer::traits::SerialPort;
use log::debug;

// CH340/CH340G USB identifiers
pub const CH340_VID: u16 = 0x1A86;
pub const CH340_PID: u16 = 0x7523;
pub const CH340K_PID: u16 = 0x5523;
pub const CH340N_PID: u16 = 0x5584;

// CH341 in UART mode
pub const CH341_UART_PID: u16 = 0x5523;

// CH347 USB identifiers
pub const CH347_VID: u16 = 0x1A86;
pub const CH347_PID: u16 = 0x55DB;

/// Discover and open the first available serial port device
pub fn discover_serial() -> Result<Box<dyn SerialPort>> {
    debug!("Discovering serial devices...");

    // Scan all USB devices
    let devices: Vec<_> = nusb::list_devices()?.collect();

    // 1. Look for CH347 devices (Interface 0)
    for device in &devices {
        if device.vendor_id() == CH347_VID && device.product_id() == CH347_PID {
            debug!("Found CH347 device");
            let dev = device.clone().open()?;
            return Ok(Box::new(ch347::Ch347Serial::new(dev)?));
        }
    }

    // 2. Look for CH340 devices (pure UART)
    for device in &devices {
        if device.vendor_id() == CH340_VID {
            let pid = device.product_id();
            if pid == CH340_PID || pid == CH340K_PID || pid == CH340N_PID {
                debug!("Found CH340 device (PID: 0x{:04X})", pid);
                let dev = device.clone().open()?;
                return Ok(Box::new(ch340::Ch340Serial::new(dev)?));
            }
        }
    }

    // 3. Look for CH341 in UART mode
    for device in &devices {
        if device.vendor_id() == CH340_VID && device.product_id() == CH341_UART_PID {
            debug!("Found CH341 in UART mode");
            let dev = device.clone().open()?;
            return Ok(Box::new(ch340::Ch340Serial::new(dev)?));
        }
    }

    Err(Error::Other(
        "No serial device found. Connect a CH340/CH341/CH347 device.".to_string(),
    ))
}

/// Re-export commonly used types
pub use ch340::Ch340Serial;
pub use ch347::Ch347Serial;
