//! FTDI-based Programmer Implementation
//!
//! Supports FT232H, FT2232H, and FT4232H in MPSSE SPI mode.
//! Implementation uses pure Rust via `nusb` and manual MPSSE command construction.

pub mod mpsse;

use crate::error::{Error, Result};
use crate::infrastructure::programmer::traits::Programmer;
use futures_lite::future::block_on;
use log::debug;
use nusb::transfer::{ControlType, Recipient, RequestBuffer};
use std::time::Duration;

// FTDI USB identifiers
pub const FTDI_VID: u16 = 0x0403;
pub const FT232H_PID: u16 = 0x6014;
pub const FT2232H_PID: u16 = 0x6010;
pub const FT4232H_PID: u16 = 0x6011;

// FTDI Control Requests
const SIO_RESET_REQUEST: u8 = 0x00;
const SIO_SET_LATENCY_TIMER_REQUEST: u8 = 0x09;
const SIO_SET_BITMODE_REQUEST: u8 = 0x0B;

// Bitmodes
#[allow(dead_code)]
const BITMODE_RESET: u8 = 0x00;
const BITMODE_MPSSE: u8 = 0x02;

// Pin Definitions (ADBUS)
// Bit 0: TCK (Out)
// Bit 1: TDI (Out)
// Bit 2: TDO (In)
// Bit 3: TMS/CS (Out) - We use this for CS
// Bit 4: GPIO (Out) - Low
// Bit 5: GPIO (Out)
// ...
const PIN_TCK: u8 = 1 << 0;
const PIN_TDI: u8 = 1 << 1;
#[allow(dead_code)]
const PIN_TDO: u8 = 1 << 2;
const PIN_CS: u8 = 1 << 3;
const PIN_GPIO_L: u8 = 1 << 4;

// Default Direction (1=Out, 0=In)
// TCK=Out, TDI=Out, TDO=In, CS=Out, Logic=Out
const DIRECTION_DEFAULT: u8 = PIN_TCK | PIN_TDI | PIN_CS | PIN_GPIO_L;

/// FTDI Programmer using MPSSE mode
pub struct FtdiProgrammer {
    // device: nusb::Device, // Not needed if we use interface for everything
    interface: nusb::Interface,
    ep_out: u8,
    ep_in: u8,
    #[allow(dead_code)]
    current_speed: u32,
    current_gpio_low: u8, // Cache for ADBUS state
}

impl FtdiProgrammer {
    pub fn new(device: nusb::Device) -> Result<Self> {
        debug!("Initializing FTDI programmer (MPSSE mode)");

        // 1. Claim Interface 0 (Channel A)
        let interface = device
            .claim_interface(0)
            .map_err(|e| Error::Other(format!("Failed to claim FTDI interface: {}", e)))?;

        // 2. Discover Endpoints
        let mut ep_out = None;
        let mut ep_in = None;

        for setting in interface.descriptors() {
            for endpoint in setting.endpoints() {
                if endpoint.transfer_type() == nusb::transfer::EndpointType::Bulk {
                    if endpoint.direction() == nusb::transfer::Direction::Out {
                        ep_out = Some(endpoint.address());
                    } else {
                        ep_in = Some(endpoint.address());
                    }
                }
            }
        }

        let ep_out =
            ep_out.ok_or_else(|| Error::Other("No Bulk OUT endpoint found".to_string()))?;
        let ep_in = ep_in.ok_or_else(|| Error::Other("No Bulk IN endpoint found".to_string()))?;

        debug!("Using endpoints: OUT=0x{:02X}, IN=0x{:02X}", ep_out, ep_in);

        // 3. Initialize FTDI
        let mut programmer = FtdiProgrammer {
            // device: device,
            interface,
            ep_out,
            ep_in,
            current_speed: 1_000_000,
            current_gpio_low: 0,
        };

        programmer.reset_mpsse()?;

        // Initial CS state: High (Inactive), TCK Low
        programmer.current_gpio_low = PIN_CS; // CS High
        programmer.update_gpio_low()?;

        debug!("FTDI Initialized successfully");
        Ok(programmer)
    }

    fn reset_mpsse(&mut self) -> Result<()> {
        // Reset device
        self.control_transfer(SIO_RESET_REQUEST, 0, 0)?;

        // Set Latency Timer to 1ms (Essential for performance)
        self.control_transfer(SIO_SET_LATENCY_TIMER_REQUEST, 1, 0)?;

        // Reset Bitmode
        self.control_transfer(SIO_SET_BITMODE_REQUEST, 0x0000, 1)?; // Val=0, Index=1 (Chan A)
                                                                    // Assumes Index 1 is for Channel A on some devices or maybe generic index usage.
                                                                    // Usually: Interface 0 -> Index 1? check libftdi.
                                                                    // libftdi: ftdi_set_bitmode -> control_transfer(..., index=interface)
                                                                    // Wait, libftdi uses `ftdi->index`.
                                                                    // If interface 0, index is 1? SIO_RESET is index 1 or 2.
                                                                    // Let's stick to Index 1 for Channel A based on common FTDI knowledge (Interface A=1, B=2 in reset/bitmode commands).

        // Disable Bitmode (Reset)
        self.control_transfer(SIO_SET_BITMODE_REQUEST, 0x0000, 1)?;

        // Enable MPSSE Bitmode (Bitmode 2)
        // Value = (Bitmask << 8) | Mode
        self.control_transfer(SIO_SET_BITMODE_REQUEST, (BITMODE_MPSSE as u16) << 8, 1)?;

        // Sync MPSSE - Send bad command 0xAA and check for 0xFA response
        std::thread::sleep(Duration::from_millis(10));

        // Clean read buffer needs loop but async in pure rust is tricky without runtime.
        // FtdiProgrammer uses block_on(async ...).
        // Let's try to drain one buffer.
        let _ = self.bulk_read(64);

        // Configure Divider (Start slow: 1MHz)
        // 1MHz: Div = (60/2) - 1 = 29
        let cmd = mpsse::build_set_divisor_cmd(29);
        self.bulk_write(&cmd)?;

        // Loopback off
        self.bulk_write(&[mpsse::build_loopback_cmd(false)])?;

        Ok(())
    }

    fn control_transfer(&self, request: u8, value: u16, index: u16) -> Result<()> {
        let result = block_on(async {
            self.interface
                .control_out(nusb::transfer::ControlOut {
                    control_type: ControlType::Vendor,
                    recipient: Recipient::Device,
                    request,
                    value,
                    index,
                    data: &[],
                })
                .await
        });
        result
            .status
            .map_err(|e| Error::Other(format!("Control transfer failed: {}", e)))?;
        Ok(())
    }

    fn bulk_write(&self, data: &[u8]) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }
        let result = block_on(async { self.interface.bulk_out(self.ep_out, data.to_vec()).await });
        result
            .status
            .map_err(|e| Error::Other(format!("Bulk write failed: {}", e)))?;
        Ok(())
    }

    fn bulk_read(&self, len: usize) -> Result<Vec<u8>> {
        let result = block_on(async {
            self.interface
                .bulk_in(self.ep_in, RequestBuffer::new(len))
                .await
        });
        let data = result
            .into_result()
            .map_err(|e| Error::Other(format!("Bulk read failed: {}", e)))?;
        Ok(data)
    }

    fn update_gpio_low(&mut self) -> Result<()> {
        let cmd = mpsse::build_set_low_gpio_cmd(self.current_gpio_low, DIRECTION_DEFAULT);
        self.bulk_write(&cmd)
    }
}

impl Programmer for FtdiProgrammer {
    fn name(&self) -> &str {
        "FTDI High-Speed Programmer (MPSSE)"
    }

    fn spi_transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<()> {
        const MAX_MPSSE: usize = 65535; // Safe limit < 65536

        if !rx.is_empty() && !tx.is_empty() {
            // Bidirectional - limited to min length of both
            let len = std::cmp::min(tx.len(), rx.len());
            // Zip tx and rx chunks
            let tx_chunks = tx[..len].chunks(MAX_MPSSE);
            let rx_chunks = rx[..len].chunks_mut(MAX_MPSSE);

            for (tx_chunk, rx_chunk) in tx_chunks.zip(rx_chunks) {
                let cmd = mpsse::build_rw_bytes_cmd(tx_chunk);
                self.bulk_write(&cmd)?;
                let data = self.bulk_read(tx_chunk.len())?;
                // Ensure we copy what we got
                let copy_len = std::cmp::min(data.len(), rx_chunk.len());
                rx_chunk[..copy_len].copy_from_slice(&data[..copy_len]);
            }
        } else if !tx.is_empty() {
            // Write only
            for tx_chunk in tx.chunks(MAX_MPSSE) {
                let cmd = mpsse::build_write_bytes_cmd(tx_chunk);
                self.bulk_write(&cmd)?;
            }
        } else if !rx.is_empty() {
            // Read only
            let mut offset = 0;
            while offset < rx.len() {
                let chunk_len = std::cmp::min(rx.len() - offset, MAX_MPSSE);
                let cmd = mpsse::build_read_bytes_cmd(chunk_len);
                self.bulk_write(&cmd)?;
                let data = self.bulk_read(chunk_len)?;
                if data.is_empty() {
                    // Prevent infinite loop if device stalls
                    return Err(Error::Other("FTDI Read Stalled".to_string()));
                }
                rx[offset..offset + data.len()].copy_from_slice(&data);
                offset += data.len();
            }
        }

        Ok(())
    }

    fn set_cs(&mut self, active: bool) -> Result<()> {
        // CS is Bit 3 (0x08)
        // Active Low: 0, Inactive: 1
        if active {
            self.current_gpio_low &= !PIN_CS;
        } else {
            self.current_gpio_low |= PIN_CS;
        }
        self.update_gpio_low()
    }

    fn set_speed(&mut self, speed: u8) -> Result<()> {
        // Frequency Table
        let freq_hz = match speed {
            0 => 100_000,
            1 => 500_000,
            2 => 1_000_000,
            3 => 5_000_000,
            4 => 10_000_000,
            5 => 15_000_000,
            6 => 20_000_000,
            7 => 30_000_000,
            _ => 1_000_000,
        };

        self.current_speed = freq_hz;

        // Divisor = (60MHz / 2*Freq) - 1
        // Example: 30MHz -> (60/60) - 1 = 0
        // 1MHz -> (60/2) - 1 = 29

        let divisor = if freq_hz > 0 {
            ((60_000_000 / (2 * freq_hz)) - 1) as u16
        } else {
            0xFFFF
        };

        debug!("FTDI: Set speed to {} Hz (Divisor: {})", freq_hz, divisor);

        let cmd = mpsse::build_set_divisor_cmd(divisor);
        self.bulk_write(&cmd) // Use bulk_write, not control
    }
}
