//! Infrastructure - Programmer Management
//!
//! This module handles programmer discovery and abstraction.

pub mod ch341a;
pub mod traits;

#[cfg(test)]
pub mod mock;

pub use ch341a::Ch341a;
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
    nusb::list_devices()?
        .find(|d| d.vendor_id() == ch341a::CH341A_VID && d.product_id() == ch341a::CH341A_PID)
        .ok_or(Error::ProgrammerNotFound)?
        .open()
        .map_err(Into::into)
}
