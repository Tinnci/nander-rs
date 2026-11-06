//! CH347 Programmer Implementation
//!
//! Supports WCH CH347T (0x1A86:0x55DB) and CH347F (0x1A86:0x55DE) in High-Speed SPI/I2C mode.

pub mod protocol;

use crate::error::{Error, Result};
use crate::infrastructure::programmer::traits::Programmer;
use futures_lite::future::block_on;
use log::{debug, trace};
use nusb::transfer::RequestBuffer;
use nusb::Device;
use protocol::SpiSpeed;

pub const CH347_VID: u16 = 0x1A86;
pub const CH347T_PID: u16 = 0x55DB;
pub const CH347F_PID: u16 = 0x55DE;
pub const CH347_HID_PID: u16 = 0x55DC;

pub struct Ch347 {
    #[allow(dead_code)]
    device: Device,
    interface: nusb::Interface,
    out_endpoint: u8,
    in_endpoint: u8,
    current_speed: SpiSpeed,
    is_ch347f: bool,
    i2c_switched: bool,
}

impl Ch347 {
    /// Create a new CH347 instance from an opened device (defaults to CH347T)
    pub fn new(device: Device) -> Result<Self> {
        Self::with_pid(device, CH347T_PID)
    }

    /// Create a new CH347 instance specifying the product ID
    pub fn with_pid(device: Device, pid: u16) -> Result<Self> {
        let is_ch347f = pid == CH347F_PID;

        let (discovered_iface, discovered_out, discovered_in) =
            Self::find_interface_and_endpoints(&device);

        debug!(
            "CH347: Discovered interface {} (OUT: 0x{:02X}, IN: 0x{:02X})",
            discovered_iface, discovered_out, discovered_in
        );

        // Try claiming the interface, with kernel driver detachment on Linux
        let candidates = [
            discovered_iface,
            protocol::MODE_1_INTERFACE,
            protocol::MODE_2_INTERFACE,
        ];

        let mut claimed = None;
        for &iface_idx in &candidates {
            if let Ok(iface) = device.detach_and_claim_interface(iface_idx) {
                claimed = Some((iface, iface_idx));
                break;
            }
            if let Ok(iface) = device.claim_interface(iface_idx) {
                claimed = Some((iface, iface_idx));
                break;
            }
        }

        let (interface, claimed_iface_num) = claimed.ok_or_else(|| {
            Error::Other(format!(
                "Failed to claim any CH347 SPI interface (tried {:?})",
                candidates
            ))
        })?;

        debug!(
            "CH347: Successfully claimed interface {}",
            claimed_iface_num
        );

        let out_endpoint = discovered_out;
        let in_endpoint = discovered_in;

        let mut programmer = Self {
            device,
            interface,
            out_endpoint,
            in_endpoint,
            current_speed: SpiSpeed::Speed60M, // Default 60MHz like SNANDer
            is_ch347f,
            i2c_switched: false,
        };

        // Initialize SPI at default high speed (60MHz)
        programmer.set_speed(SpiSpeed::Speed60M.to_u8())?;

        Ok(programmer)
    }

    /// Automatically find the vendor SPI bulk interface and endpoints
    fn find_interface_and_endpoints(device: &Device) -> (u8, u8, u8) {
        for cfg in device.configurations() {
            for iface in cfg.interfaces() {
                for alt in iface.alt_settings() {
                    let mut out_ep = None;
                    let mut in_ep = None;

                    for ep in alt.endpoints() {
                        if ep.transfer_type() == nusb::transfer::EndpointType::Bulk {
                            if ep.direction() == nusb::transfer::Direction::Out {
                                out_ep = Some(ep.address());
                            } else {
                                in_ep = Some(ep.address());
                            }
                        }
                    }

                    if let (Some(out_addr), Some(in_addr)) = (out_ep, in_ep) {
                        // Standard CH347 SPI endpoints are 0x06 and 0x86
                        if out_addr == protocol::DEFAULT_WRITE_EP
                            && in_addr == protocol::DEFAULT_READ_EP
                        {
                            return (alt.interface_number(), out_addr, in_addr);
                        }
                        if alt.interface_number() == protocol::MODE_1_INTERFACE {
                            return (protocol::MODE_1_INTERFACE, out_addr, in_addr);
                        }
                    }
                }
            }
        }

        (
            protocol::MODE_1_INTERFACE,
            protocol::DEFAULT_WRITE_EP,
            protocol::DEFAULT_READ_EP,
        )
    }

    /// Send a bulk write to the CH347 OUT endpoint
    fn bulk_write(&self, data: &[u8]) -> Result<()> {
        trace!("CH347 OUT: {:02X?}", data);
        let result = block_on(async {
            self.interface
                .bulk_out(self.out_endpoint, data.to_vec())
                .await
        });
        result
            .status
            .map_err(|e| Error::Other(format!("CH347 bulk_out error: {}", e)))?;
        Ok(())
    }

    /// Read data from the CH347 IN endpoint
    fn bulk_read(&self, len: usize) -> Result<Vec<u8>> {
        let result = block_on(async {
            self.interface
                .bulk_in(self.in_endpoint, RequestBuffer::new(len))
                .await
        });
        let data = result
            .into_result()
            .map_err(|e| Error::Other(format!("CH347 bulk_in error: {}", e)))?;
        trace!("CH347 IN: {:02X?}", data);
        Ok(data)
    }

    fn i2c_switch_if_needed(&mut self) -> Result<()> {
        if self.is_ch347f && !self.i2c_switched {
            let switch_cmd = vec![
                protocol::CMD_CH347F_SWITCH,
                0x08,
                0x00,
                0x00,
                0x00,
                0x81,
                0x81,
                0x00,
                0x00,
                0x00,
                0x00,
            ];
            self.bulk_write(&switch_cmd)?;
            let _ = self.bulk_read(4);
            self.i2c_switched = true;
        }
        Ok(())
    }
}

impl Programmer for Ch347 {
    fn name(&self) -> &str {
        if self.is_ch347f {
            "CH347F High-Speed Programmer"
        } else {
            "CH347 High-Speed Programmer"
        }
    }

    fn probe(&mut self) -> Result<()> {
        // Re-apply SPI config and check response to ensure hardware is responsive
        let packet = protocol::build_set_cfg_cmd(self.current_speed);
        self.bulk_write(&packet)?;
        let resp = self.bulk_read(4)?;
        if resp.is_empty() || resp[0] != protocol::CMD_SPI_SET_CFG {
            return Err(Error::Other("CH347 probe failed".to_string()));
        }
        Ok(())
    }

    fn set_cs(&mut self, active: bool) -> Result<()> {
        let packet = protocol::build_cs_cmd(active);
        self.bulk_write(&packet)
    }

    fn set_speed(&mut self, speed: u8) -> Result<()> {
        let spi_speed = protocol::SpiSpeed::from_u8(speed);
        debug!("CH347: Setting speed to {}", spi_speed.description());

        let packet = protocol::build_set_cfg_cmd(spi_speed);
        self.bulk_write(&packet)?;
        let resp = self.bulk_read(4)?;
        if resp.len() >= 4 && resp[0] == protocol::CMD_SPI_SET_CFG && resp[3] != 0 {
            return Err(Error::Other(format!(
                "CH347: SPI configure failed with error code 0x{:02X}",
                resp[3]
            )));
        }
        self.current_speed = spi_speed;
        Ok(())
    }

    fn spi_write(&mut self, data: &[u8]) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }

        // Chunk writes into MAX_DATA_LEN (507 bytes)
        for chunk in data.chunks(protocol::CH347_MAX_DATA_LEN) {
            let packet = protocol::build_spi_write_packet(chunk);
            self.bulk_write(&packet)?;
            let resp = self.bulk_read(4)?;
            if resp.is_empty() || resp[0] != protocol::CMD_SPI_BLCK_WR {
                return Err(Error::Other(format!(
                    "CH347: SPI write command failed (response: {:02X?})",
                    resp
                )));
            }
        }

        Ok(())
    }

    fn spi_read(&mut self, len: usize) -> Result<Vec<u8>> {
        if len == 0 {
            return Ok(Vec::new());
        }

        // Safely chunk large reads in 32KB blocks to prevent timeout / overrun
        const CHUNK_LIMIT: usize = 32 * 1024;
        let mut result = Vec::with_capacity(len);
        let mut remaining = len;

        while remaining > 0 {
            let read_size = remaining.min(CHUNK_LIMIT);
            let cmd = protocol::build_spi_read_cmd(read_size);
            self.bulk_write(&cmd)?;

            let mut chunk_received = 0;
            while chunk_received < read_size {
                let to_read = read_size - chunk_received;
                let expected_packet_len = (to_read + 3).min(protocol::CH347_PACKET_SIZE);
                let resp = self.bulk_read(expected_packet_len)?;
                if resp.len() < 3 {
                    return Err(Error::Other(
                        "CH347: Truncated response packet from device".to_string(),
                    ));
                }
                if resp[0] != protocol::CMD_SPI_BLCK_RD {
                    return Err(Error::Other(format!(
                        "CH347: Read error, unexpected response byte: 0x{:02X} (expected 0x{:02X})",
                        resp[0], protocol::CMD_SPI_BLCK_RD
                    )));
                }

                let data_len = (resp[1] as usize) | ((resp[2] as usize) << 8);
                if resp.len() < 3 + data_len {
                    return Err(Error::Other(format!(
                        "CH347: Incomplete packet: expected {} payload bytes, got {}",
                        data_len,
                        resp.len() - 3
                    )));
                }

                result.extend_from_slice(&resp[3..3 + data_len]);
                chunk_received += data_len;
            }

            remaining -= read_size;
        }

        Ok(result)
    }

    fn spi_read_bulk(&mut self, len: usize) -> Result<Vec<u8>> {
        // ch347_read already performs high-speed streaming transfers natively
        self.spi_read(len)
    }

    fn spi_transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<()> {
        if tx.is_empty() && rx.is_empty() {
            return Ok(());
        }
        if rx.is_empty() {
            return self.spi_write(tx);
        }
        if tx.is_empty() {
            let data = self.spi_read(rx.len())?;
            rx.copy_from_slice(&data);
            return Ok(());
        }

        let common_len = tx.len().min(rx.len());
        for (tx_chunk, rx_chunk) in tx[..common_len]
            .chunks(protocol::CH347_MAX_DATA_LEN)
            .zip(rx[..common_len].chunks_mut(protocol::CH347_MAX_DATA_LEN))
        {
            let packet = protocol::build_spi_transfer_packet(tx_chunk);
            self.bulk_write(&packet)?;

            let resp = self.bulk_read(tx_chunk.len() + 3)?;
            if resp.len() < 3 || resp[0] != protocol::CMD_SPI_RD_WR {
                return Err(Error::Other(format!(
                    "CH347: Full-duplex transfer error, unexpected response: {:02X?}",
                    resp
                )));
            }

            let data_len = (resp[1] as usize) | ((resp[2] as usize) << 8);
            let copy_len = data_len.min(rx_chunk.len());
            rx_chunk[..copy_len].copy_from_slice(&resp[3..3 + copy_len]);
        }

        if tx.len() > common_len {
            self.spi_write(&tx[common_len..])?;
        }
        if rx.len() > common_len {
            let data = self.spi_read(rx.len() - common_len)?;
            rx[common_len..].copy_from_slice(&data);
        }

        Ok(())
    }

    fn spi_transaction(&mut self, tx: &[u8], rx_len: usize) -> Result<Vec<u8>> {
        self.set_cs(true)?;
        let result = (|| {
            if !tx.is_empty() {
                self.spi_write(tx)?;
            }
            if rx_len > 0 {
                self.spi_read(rx_len)
            } else {
                Ok(Vec::new())
            }
        })();
        let _ = self.set_cs(false);
        result
    }

    fn spi_transaction_write(&mut self, tx: &[u8]) -> Result<()> {
        self.set_cs(true)?;
        let result = self.spi_write(tx);
        let _ = self.set_cs(false);
        result
    }

    fn max_bulk_transfer_size(&self) -> usize {
        64 * 1024
    }

    fn i2c_write(&mut self, addr: u8, data: &[u8]) -> Result<()> {
        self.i2c_switch_if_needed()?;

        if data.len() > 31 {
            return Err(Error::InvalidParameter(
                "I2C write data length exceeds 31 bytes".to_string(),
            ));
        }

        let mut cmd = Vec::with_capacity(data.len() + 6);
        cmd.push(protocol::CMD_I2C_STREAM);
        cmd.push(protocol::i2c_sub::START);
        cmd.push(protocol::i2c_sub::OUT | (data.len() as u8 + 1));
        cmd.push(addr); // Device address (Write)
        cmd.extend_from_slice(data);
        cmd.push(protocol::i2c_sub::STOP);
        cmd.push(protocol::i2c_sub::END);

        self.bulk_write(&cmd)?;
        // Read ACK responses from IN endpoint to keep buffer clear
        let _ = self.bulk_read(data.len() + 1);
        Ok(())
    }

    fn i2c_read(&mut self, addr: u8, len: usize) -> Result<Vec<u8>> {
        self.i2c_switch_if_needed()?;

        if len > 32 {
            return Err(Error::InvalidParameter(
                "I2C read length exceeds 32 bytes".to_string(),
            ));
        }

        let mut cmd = Vec::with_capacity(8);
        cmd.push(protocol::CMD_I2C_STREAM);
        cmd.push(protocol::i2c_sub::START);
        cmd.push(protocol::i2c_sub::OUT | 1);
        cmd.push(addr | 1); // Device address (Read)
        cmd.push(protocol::i2c_sub::IN | (len as u8));
        cmd.push(protocol::i2c_sub::STOP);
        cmd.push(protocol::i2c_sub::END);

        self.bulk_write(&cmd)?;
        self.bulk_read(len)
    }
}
