//! Infrastructure - Programmer Management
//!
//! This module handles programmer discovery and abstraction.

pub mod ch341a;
pub mod device_database;
pub mod simulator;
pub mod traits;

#[cfg(test)]
pub mod mock;

pub use ch341a::Ch341a;
pub use device_database::{DeviceCompatibility, DeviceInfo, WchDeviceDatabase};
pub use traits::Programmer;

use crate::error::{Error, Result};
use log::debug;

/// Find and open the first available programmer
pub fn discover() -> Result<Box<dyn Programmer>> {
    debug!("Starting programmer discovery...");

    // Try CH341A
    if let Ok(device) = find_ch341a() {
        debug!("Found CH341A programmer");
        let p = Ch341a::new(device)?;
        return Ok(Box::new(p));
    }

    // Add other programmers here...

    Err(Error::ProgrammerNotFound)
}

fn find_ch341a() -> Result<nusb::Device> {
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
        for (_device, info) in &wch_devices {
            debug!("  {}", info);
            if let Some(help) = info.help_message {
                // Print help message indented
                for line in help.lines() {
                    debug!("    {}", line);
                }
            }
        }
    } else {
        debug!("No WCH devices found on this system");
        debug!("Supported devices: CH341A (VID:PID = 1A86:5512)");
    }

    // Look for a supported programmer
    let supported_device = wch_devices
        .into_iter()
        .find(|(_, info)| info.compatibility == device_database::DeviceCompatibility::Supported);

    match supported_device {
        Some((device, info)) => {
            debug!("✓ Using: {}", info.name);
            match device.open() {
                Ok(d) => Ok(d),
                Err(e) => {
                    // Enhanced error message for common Windows driver issues
                    let error_str = e.to_string();

                    let enhanced_msg = if error_str.contains("CH341")
                        || error_str.contains("driver")
                    {
                        format!(
                            "Device found but cannot connect: {}\n\n\
                            ⚠ DRIVER ISSUE DETECTED\n\
                            Your CH341A is using an incompatible driver (likely 'CH341_A64' or similar).\n\n\
                            SOLUTION - Install WinUSB Driver:\n\
                            1. Download Zadig: https://zadig.akeo.ie/\n\
                            2. Run Zadig as Administrator\n\
                            3. Options → List All Devices ✓\n\
                            4. Select 'USB-SERIAL CH341A' or 'Interface 0' from dropdown\n\
                            5. Driver box should show current driver (e.g., CH341_A64)\n\
                            6. Click the arrows to select 'WinUSB' as replacement driver\n\
                            7. Click 'Replace Driver' or 'Install Driver'\n\
                            8. Wait for completion, then replug your device\n\n\
                            After driver replacement, nander-rs will work correctly.\n\
                            Note: This won't break other CH341 software - you can switch back anytime.",
                            e
                        )
                    } else if error_str.contains("Access") || error_str.contains("denied") {
                        format!(
                            "Device found but access denied: {}\n\n\
                            SOLUTIONS:\n\
                            1. Run as Administrator (Windows)\n\
                            2. Check if another program is using the device\n\
                            3. Try a different USB port\n\
                            4. Restart your computer",
                            e
                        )
                    } else {
                        format!("Failed to open device: {}", e)
                    };

                    Err(Error::Other(enhanced_msg))
                }
            }
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
                    "WCH device(s) found, but none in supported mode.\n\
                    Check the debug log above for device details and troubleshooting steps."
                        .to_string()
                } else {
                    format!(
                        "No WCH programmer detected.\n\
                        Found {} other USB device(s), but none are CH341A programmers.\n\
                        Please connect a CH341A device in SPI mode (PID 0x5512).",
                        all_devices.len()
                    )
                }
            };

            Err(Error::Other(error_msg))
        }
    }
}
