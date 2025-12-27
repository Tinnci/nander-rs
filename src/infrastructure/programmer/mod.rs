//! Infrastructure - Programmer Management
//!
//! This module handles programmer discovery and abstraction.

pub mod ch341a;
pub mod ch347;
pub mod device_database;
pub mod simulator;
pub mod traits;

#[cfg(test)]
pub mod mock;

pub use ch341a::Ch341a;
pub use ch347::Ch347;
pub use device_database::{DeviceCompatibility, DeviceInfo, WchDeviceDatabase};
pub use traits::Programmer;

use crate::error::{Error, Result};
use log::debug;

/// Find and open the first available programmer
pub fn discover() -> Result<Box<dyn Programmer>> {
    debug!("Starting programmer discovery...");

    // Find a supported USB device (returns DeviceInfo and its PID)
    let (device_info, pid) = find_supported_device()?;

    // Open the device
    let device = device_info
        .open()
        .map_err(|e| Error::Other(format!("Failed to open device: {}", e)))?;

    // Create appropriate programmer based on PID
    match pid {
        0x5512 => {
            debug!("Initializing CH341A programmer");
            let p = Ch341a::new(device)?;
            Ok(Box::new(p))
        }
        0x55DB => {
            debug!("Initializing CH347 programmer (High-Speed)");
            let p = Ch347::new(device)?;
            Ok(Box::new(p))
        }
        _ => Err(Error::ProgrammerNotFound),
    }
}

fn find_supported_device() -> Result<(nusb::DeviceInfo, u16)> {
    use device_database::WchDeviceDatabase;

    // Scan all USB devices
    let all_devices: Vec<_> = nusb::list_devices()?.collect();

    debug!("Scanning {} USB devices...", all_devices.len());

    // Filter WCH devices and analyze them
    let wch_devices: Vec<_> = all_devices
        .iter()
        .filter(|d| d.vendor_id() == WchDeviceDatabase::WCH_VID)
        .map(|d| {
            (
                d,
                WchDeviceDatabase::identify(d.vendor_id(), d.product_id()),
            )
        })
        .collect();

    if !wch_devices.is_empty() {
        debug!("Found {} WCH device(s):", wch_devices.len());
        for (device, info) in &wch_devices {
            debug!("  {} (PID: 0x{:04X})", info, device.product_id());
            if let Some(help) = info.help_message {
                // Print help message indented
                for line in help.lines() {
                    debug!("    {}", line);
                }
            }
        }
    } else {
        debug!("No WCH devices found on this system");
        debug!("Supported devices: CH341A (1A86:5512), CH347 (1A86:55DB)");
    }

    // Look for a supported programmer
    let supported_device = wch_devices
        .into_iter()
        .find(|(_, info)| info.compatibility == DeviceCompatibility::Supported);

    match supported_device {
        Some((device, info)) => {
            debug!("✓ Using: {}", info.name);
            Ok((device.clone(), device.product_id()))
        }
        None => {
            // Build detailed error message
            let error_msg = if all_devices.is_empty() {
                "No USB devices detected. Check USB connection.".to_string()
            } else {
                let wch_count = all_devices
                    .iter()
                    .filter(|d| d.vendor_id() == WchDeviceDatabase::WCH_VID)
                    .count();

                if wch_count > 0 {
                    "WCH device(s) found, but none in supported mode.\\n\\\
                    Check the debug log above for device details and troubleshooting steps."
                        .to_string()
                } else {
                    format!(
                        "No WCH programmer detected.\\n\\\
                        Found {} other USB device(s), but none are supported programmers.\\n\\\
                        Please connect a CH341A (1A86:5512) or CH347 (1A86:55DB) device.",
                        all_devices.len()
                    )
                }
            };

            Err(Error::Other(error_msg))
        }
    }
}
