//! CH341A programmer driver
//!
//! This module implements the CH341A USB-to-SPI bridge driver
//! using the nusb library for pure Rust USB communication.

mod protocol;

use futures_lite::future::block_on;
use log::{debug, trace};
use nusb::transfer::{ControlType, Recipient, RequestBuffer};

use crate::error::{Error, Result};
use crate::hardware::Programmer;

// CH341A USB identifiers
const CH341A_VID: u16 = 0x1A86;
const CH341A_PID: u16 = 0x5512;

// CH341A endpoints
const EP_OUT: u8 = 0x02;
const EP_IN: u8 = 0x82;

/// CH341A programmer instance
pub struct Ch341a {
    device: nusb::Device,
    interface: nusb::Interface,
    speed: protocol::SpiSpeed,
}

impl Ch341a {
    /// Configure SPI mode
    fn configure_spi(&mut self) -> Result<()> {
        debug!("Configuring CH341A for SPI mode...");

        // Set SPI mode parameters
        let cmd = protocol::build_set_mode_cmd(self.speed);
        self.bulk_write(&cmd)?;

        debug!("SPI mode configured");
        Ok(())
    }

    /// Perform a bulk write to the device
    fn bulk_write(&self, data: &[u8]) -> Result<()> {
        trace!("USB OUT: {:02X?}", data);

        let result = block_on(async { self.interface.bulk_out(EP_OUT, data.to_vec()).await });

        result.status?;
        Ok(())
    }

    /// Perform a bulk read from the device
    fn bulk_read(&self, len: usize) -> Result<Vec<u8>> {
        let result =
            block_on(async { self.interface.bulk_in(EP_IN, RequestBuffer::new(len)).await });

        let data = result.into_result()?;
        trace!("USB IN: {:02X?}", data);
        Ok(data)
    }

    /// Send a USB control transfer
    #[allow(dead_code)]
    fn control_transfer(&self, request: u8, value: u16, index: u16) -> Result<()> {
        use nusb::transfer::ControlOut;

        let control_out = ControlOut {
            control_type: ControlType::Vendor,
            recipient: Recipient::Device,
            request,
            value,
            index,
            data: &[],
        };

        block_on(async { self.interface.control_out(control_out).await }).into_result()?;

        Ok(())
    }
}

impl Programmer for Ch341a {
    fn open() -> Result<Self> {
        debug!(
            "Searching for CH341A device (VID={:04X}, PID={:04X})...",
            CH341A_VID, CH341A_PID
        );

        // Find the device
        let device_info = nusb::list_devices()?
            .find(|d| d.vendor_id() == CH341A_VID && d.product_id() == CH341A_PID)
            .ok_or(Error::ProgrammerNotFound)?;

        debug!("Found device: {:?}", device_info);

        // Open the device
        let device = device_info.open()?;

        // Claim the interface
        let interface = device.claim_interface(0)?;

        let mut ch341a = Ch341a {
            device,
            interface,
            speed: protocol::SpiSpeed::Medium,
        };

        // Initialize for SPI mode
        ch341a.configure_spi()?;

        Ok(ch341a)
    }

    fn close(&mut self) -> Result<()> {
        debug!("Closing CH341A connection");
        // nusb handles cleanup automatically when dropped
        Ok(())
    }

    fn spi_transfer(&mut self, tx_data: &[u8], rx_data: &mut [u8]) -> Result<()> {
        assert_eq!(
            tx_data.len(),
            rx_data.len(),
            "TX and RX buffers must be same length"
        );

        // Build the SPI command packet
        let cmd = protocol::build_spi_transfer_cmd(tx_data);
        self.bulk_write(&cmd)?;

        // Read the response
        let response = self.bulk_read(rx_data.len())?;
        rx_data.copy_from_slice(&response);

        Ok(())
    }

    fn set_cs(&mut self, active: bool) -> Result<()> {
        let cmd = protocol::build_cs_cmd(active);
        self.bulk_write(&cmd)?;
        Ok(())
    }

    fn set_gpio(&mut self, pin: u8, level: bool) -> Result<()> {
        debug!("Setting GPIO{} = {}", pin, level);

        let cmd = protocol::build_gpio_cmd(pin, level);
        self.bulk_write(&cmd)?;

        Ok(())
    }
}

impl Drop for Ch341a {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
