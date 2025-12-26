//! Command implementations
//!
//! This module contains the actual implementation of each CLI command.

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info, warn};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use crate::database::{self, FlashType};
use crate::flash::nand::SpiNand;
use crate::flash::nor::SpiNor;
use crate::flash::{FlashOps, NorFlash};
use crate::hardware::ch341a::Ch341a;
use crate::hardware::Programmer;

/// Info command - detect and display flash chip information
pub fn info() -> Result<()> {
    info!("Searching for programmer...");

    let programmer = Ch341a::open().context("Failed to open CH341A programmer")?;

    let jedec_id = {
        let mut p = programmer;
        let id = p.read_jedec_id()?;
        drop(p);
        id
    };

    println!(
        "JEDEC ID: {:02X} {:02X} {:02X}",
        jedec_id[0], jedec_id[1], jedec_id[2]
    );

    // Look up chip in database
    match database::lookup_chip(&jedec_id) {
        Some(chip) => {
            println!("Detected: {}", chip.name);
            println!("  Manufacturer: {}", chip.manufacturer);
            println!("  Type: {:?}", chip.flash_type);
            println!(
                "  Capacity: {} bytes ({} MB)",
                chip.capacity,
                chip.capacity / 1024 / 1024
            );
            println!("  Page Size: {} bytes", chip.page_size);
            if let Some(oob) = chip.oob_size {
                println!("  OOB Size: {} bytes", oob);
            }
            if let Some(block_size) = chip.block_size {
                println!(
                    "  Block Size: {} bytes ({} KB)",
                    block_size,
                    block_size / 1024
                );
            }
            if let Some(block_count) = chip.block_count {
                println!("  Block Count: {}", block_count);
            }

            // For NAND chips, show ECC status
            if chip.flash_type == FlashType::Nand {
                let programmer = Ch341a::open().context("Failed to reopen programmer")?;
                let mut flash = SpiNand::new(programmer, chip.clone());

                let (status_reg, config_reg) = flash.get_status_registers()?;
                println!("  Status Register 1: 0x{:02X}", status_reg);
                println!("  Status Register 2: 0x{:02X}", config_reg);

                let ecc_enabled = flash.is_ecc_enabled()?;
                println!(
                    "  ECC: {}",
                    if ecc_enabled { "Enabled" } else { "Disabled" }
                );
            }
        }
        None => {
            warn!("Unknown chip! Consider adding it to the database.");
            println!("Hint: Run 'nander list' to see all supported chips");
        }
    }

    Ok(())
}

/// List command - display all supported chips
pub fn list() -> Result<()> {
    let chips = database::list_supported_chips();

    println!("Supported Flash Chips ({} total):\n", chips.len());

    // Group by flash type
    let nand_chips: Vec<_> = chips
        .iter()
        .filter(|c| c.flash_type == FlashType::Nand)
        .collect();
    let nor_chips: Vec<_> = chips
        .iter()
        .filter(|c| c.flash_type == FlashType::Nor)
        .collect();

    println!("=== SPI NAND Flash ({}) ===", nand_chips.len());
    for (i, chip) in nand_chips.iter().enumerate() {
        println!(
            "{:3}. {:24} {:12} [{:02X} {:02X} {:02X}] {:>4} MB",
            i + 1,
            chip.name,
            chip.manufacturer,
            chip.jedec_id[0],
            chip.jedec_id[1],
            chip.jedec_id[2],
            chip.capacity / 1024 / 1024
        );
    }

    println!("\n=== SPI NOR Flash ({}) ===", nor_chips.len());
    for (i, chip) in nor_chips.iter().enumerate() {
        println!(
            "{:3}. {:24} {:12} [{:02X} {:02X} {:02X}] {:>4} MB",
            i + 1,
            chip.name,
            chip.manufacturer,
            chip.jedec_id[0],
            chip.jedec_id[1],
            chip.jedec_id[2],
            chip.capacity / 1024 / 1024
        );
    }

    Ok(())
}

/// Read command - read flash contents to a file
pub fn read(output: &Path, length: Option<u32>, start: u32, disable_ecc: bool) -> Result<()> {
    info!("Opening programmer...");
    let programmer = Ch341a::open().context("Failed to open CH341A programmer")?;

    let jedec_id = {
        let mut p = programmer;
        let id = p.read_jedec_id()?;
        // Reopen for actual use
        drop(p);
        id
    };

    let chip = database::lookup_chip(&jedec_id).context("Unknown chip, cannot determine size")?;
    let programmer = Ch341a::open().context("Failed to reopen CH341A programmer")?;

    let read_length = length.unwrap_or(chip.capacity - start);

    info!(
        "Reading {} bytes ({:.2} MB) from address 0x{:08X}...",
        read_length,
        read_length as f64 / 1024.0 / 1024.0,
        start
    );

    let pb = ProgressBar::new(read_length as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
        .progress_chars("█▓▒░"));

    // Open output file
    let file = File::create(output).context("Failed to create output file")?;
    let mut writer = BufWriter::new(file);

    match chip.flash_type {
        FlashType::Nand => {
            let mut flash = SpiNand::new(programmer, chip.clone());

            // Handle ECC setting
            if disable_ecc {
                info!("Disabling internal ECC for raw read");
                flash.disable_ecc()?;
            } else {
                info!("Using internal ECC");
            }

            let page_size = chip.page_size as usize;
            let total_pages = (read_length as usize + page_size - 1) / page_size;
            let start_page = start / chip.page_size;

            let mut buffer = vec![0u8; page_size];
            let mut bytes_read = 0u32;

            for page in 0..total_pages as u32 {
                flash.read_page(start_page + page, &mut buffer)?;

                let bytes_to_write = ((read_length - bytes_read) as usize).min(page_size);
                writer.write_all(&buffer[..bytes_to_write])?;

                bytes_read += bytes_to_write as u32;
                pb.set_position(bytes_read as u64);
            }
        }
        FlashType::Nor => {
            let mut flash = SpiNor::new(programmer, chip.clone());

            // Read in chunks for better progress display
            const CHUNK_SIZE: usize = 4096;
            let mut buffer = vec![0u8; CHUNK_SIZE];
            let mut bytes_read = 0u32;
            let mut current_addr = start;

            while bytes_read < read_length {
                let remaining = (read_length - bytes_read) as usize;
                let chunk_size = remaining.min(CHUNK_SIZE);

                flash.read(current_addr, &mut buffer[..chunk_size])?;
                writer.write_all(&buffer[..chunk_size])?;

                bytes_read += chunk_size as u32;
                current_addr += chunk_size as u32;
                pb.set_position(bytes_read as u64);
            }
        }
    }

    writer.flush()?;
    pb.finish_with_message("Read complete");
    info!("Data written to {:?}", output);

    Ok(())
}

/// Write command - write a file to flash
pub fn write(input: &Path, start: u32, verify: bool, disable_ecc: bool) -> Result<()> {
    info!("Opening programmer...");
    let programmer = Ch341a::open().context("Failed to open CH341A programmer")?;

    let jedec_id = {
        let mut p = programmer;
        let id = p.read_jedec_id()?;
        drop(p);
        id
    };

    let chip = database::lookup_chip(&jedec_id).context("Unknown chip")?;
    let programmer = Ch341a::open().context("Failed to reopen CH341A programmer")?;

    // Read input file
    let file = File::open(input).context("Failed to open input file")?;
    let file_size = file.metadata()?.len() as usize;
    let mut reader = BufReader::new(file);

    info!(
        "Writing {} bytes ({:.2} MB) to address 0x{:08X}...",
        file_size,
        file_size as f64 / 1024.0 / 1024.0,
        start
    );

    let pb = ProgressBar::new(file_size as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
        .progress_chars("█▓▒░"));

    // Store disable_ecc for verify
    let verify_disable_ecc = disable_ecc;

    match chip.flash_type {
        FlashType::Nand => {
            let mut flash = SpiNand::new(programmer, chip.clone());

            // Handle ECC setting
            if disable_ecc {
                info!("Disabling internal ECC for raw write");
                flash.disable_ecc()?;
            } else {
                info!("Using internal ECC");
            }

            let page_size = chip.page_size as usize;
            let start_page = start / chip.page_size;

            let mut buffer = vec![0u8; page_size];
            let mut bytes_written = 0usize;
            let mut page = 0u32;

            loop {
                let bytes_read = reader.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }

                // Pad with 0xFF if partial page
                if bytes_read < page_size {
                    buffer[bytes_read..].fill(0xFF);
                }

                flash.write_page(start_page + page, &buffer)?;

                bytes_written += bytes_read;
                page += 1;
                pb.set_position(bytes_written as u64);
            }
        }
        FlashType::Nor => {
            let mut flash = SpiNor::new(programmer, chip.clone());

            // For NOR, we need to erase before writing
            info!("Erasing flash before writing...");
            let block_size = chip.block_size.unwrap_or(65536);
            let blocks_to_erase =
                ((file_size as u32 + start % block_size) + block_size - 1) / block_size;
            let erase_start = (start / block_size) * block_size;

            for block in 0..blocks_to_erase {
                let addr = erase_start + block * block_size;
                debug!("Erasing block at 0x{:08X}", addr);
                flash.erase_block(addr)?;
            }

            // Now write the data
            let mut data = Vec::with_capacity(file_size);
            reader.read_to_end(&mut data)?;

            flash.write(start, &data)?;
            pb.set_position(file_size as u64);
        }
    }

    pb.finish_with_message("Write complete");

    if verify {
        info!("Verifying...");
        verify_internal(input, start, verify_disable_ecc)?;
        info!("Verification successful");
    }

    Ok(())
}

/// Erase command - erase flash contents
pub fn erase(length: Option<u32>, start: u32, disable_ecc: bool) -> Result<()> {
    info!("Opening programmer...");
    let programmer = Ch341a::open().context("Failed to open CH341A programmer")?;

    let jedec_id = {
        let mut p = programmer;
        let id = p.read_jedec_id()?;
        drop(p);
        id
    };

    let chip = database::lookup_chip(&jedec_id).context("Unknown chip")?;
    let programmer = Ch341a::open().context("Failed to reopen CH341A programmer")?;

    let block_size = chip.block_size.unwrap_or(65536);
    let erase_length = length.unwrap_or(chip.capacity - start);
    let blocks = (erase_length + block_size - 1) / block_size;

    info!(
        "Erasing {} bytes ({} blocks) from address 0x{:08X}...",
        erase_length, blocks, start
    );

    let pb = ProgressBar::new(blocks as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} blocks ({eta})")?
        .progress_chars("█▓▒░"));

    match chip.flash_type {
        FlashType::Nand => {
            let mut flash = SpiNand::new(programmer, chip.clone());

            // Handle ECC setting (some chips may behave differently during erase)
            if disable_ecc {
                info!("Disabling internal ECC");
                flash.disable_ecc()?;
            }

            let start_block = start / block_size;

            for block in 0..blocks {
                let addr = (start_block + block) * block_size;
                flash.erase_block(addr)?;
                pb.inc(1);
            }
        }
        FlashType::Nor => {
            let mut flash = SpiNor::new(programmer, chip.clone());
            let start_block = (start / block_size) * block_size;

            for block in 0..blocks {
                let addr = start_block + block * block_size;
                flash.erase_block(addr)?;
                pb.inc(1);
            }
        }
    }

    pb.finish_with_message("Erase complete");
    info!("Erase complete");
    Ok(())
}

/// Verify command - verify flash contents against a file
pub fn verify(input: &Path, start: u32, disable_ecc: bool) -> Result<()> {
    info!("Verifying flash contents...");
    verify_internal(input, start, disable_ecc)
}

/// Internal verify implementation
fn verify_internal(input: &Path, start: u32, disable_ecc: bool) -> Result<()> {
    let programmer = Ch341a::open().context("Failed to open CH341A programmer")?;

    let jedec_id = {
        let mut p = programmer;
        let id = p.read_jedec_id()?;
        drop(p);
        id
    };

    let chip = database::lookup_chip(&jedec_id).context("Unknown chip")?;
    let programmer = Ch341a::open().context("Failed to reopen CH341A programmer")?;

    let expected = std::fs::read(input).context("Failed to read input file")?;
    let file_size = expected.len();

    info!(
        "Verifying {} bytes from address 0x{:08X}...",
        file_size, start
    );

    let pb = ProgressBar::new(file_size as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes}",
            )?
            .progress_chars("█▓▒░"),
    );

    match chip.flash_type {
        FlashType::Nand => {
            let mut flash = SpiNand::new(programmer, chip.clone());

            // Handle ECC setting
            if disable_ecc {
                flash.disable_ecc()?;
            }
            let page_size = chip.page_size as usize;
            let start_page = start / chip.page_size;
            let total_pages = (file_size + page_size - 1) / page_size;

            let mut buffer = vec![0u8; page_size];
            let mut bytes_verified = 0usize;

            for page in 0..total_pages as u32 {
                flash.read_page(start_page + page, &mut buffer)?;

                let expected_start = bytes_verified;
                let expected_end = (bytes_verified + page_size).min(file_size);
                let expected_slice = &expected[expected_start..expected_end];

                for (i, (a, b)) in expected_slice.iter().zip(buffer.iter()).enumerate() {
                    if a != b {
                        anyhow::bail!(
                            "Verification failed at address 0x{:08X}: expected 0x{:02X}, got 0x{:02X}",
                            start + (bytes_verified + i) as u32,
                            a, b
                        );
                    }
                }

                bytes_verified = expected_end;
                pb.set_position(bytes_verified as u64);
            }
        }
        FlashType::Nor => {
            let mut flash = SpiNor::new(programmer, chip.clone());

            const CHUNK_SIZE: usize = 4096;
            let mut buffer = vec![0u8; CHUNK_SIZE];
            let mut bytes_verified = 0usize;
            let mut current_addr = start;

            while bytes_verified < file_size {
                let remaining = file_size - bytes_verified;
                let chunk_size = remaining.min(CHUNK_SIZE);

                flash.read(current_addr, &mut buffer[..chunk_size])?;

                for (i, (a, b)) in expected[bytes_verified..bytes_verified + chunk_size]
                    .iter()
                    .zip(buffer.iter())
                    .enumerate()
                {
                    if a != b {
                        anyhow::bail!(
                            "Verification failed at address 0x{:08X}: expected 0x{:02X}, got 0x{:02X}",
                            current_addr + i as u32,
                            a, b
                        );
                    }
                }

                bytes_verified += chunk_size;
                current_addr += chunk_size as u32;
                pb.set_position(bytes_verified as u64);
            }
        }
    }

    pb.finish_with_message("Verification complete");
    info!("Verification successful - all {} bytes match", file_size);
    Ok(())
}
