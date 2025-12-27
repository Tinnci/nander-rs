use crate::application::use_cases::detect_chip::DetectChipUseCase;
use crate::application::use_cases::erase_flash::{EraseFlashUseCase, EraseParams};
use crate::application::use_cases::read_flash::{ReadFlashUseCase, ReadParams};
use crate::application::use_cases::write_flash::{WriteFlashUseCase, WriteParams};
use crate::domain::{BadBlockStrategy, FlashType, OobMode};
use crate::infrastructure::chip_database::registry::ChipRegistry;
use crate::infrastructure::flash_protocol::eeprom::{I2cEeprom, MicrowireEeprom, SpiEeprom};
use crate::infrastructure::flash_protocol::nand::SpiNand;
use crate::infrastructure::flash_protocol::nor::SpiNor;
use crate::infrastructure::programmer::Programmer;
use crate::presentation::gui::messages::{GuiMessage, WorkerMessage};
use std::fs::File;
use std::io::Write;
use std::sync::mpsc::{Receiver, Sender};

pub fn run_worker(rx: Receiver<GuiMessage>, tx: Sender<WorkerMessage>) {
    // Worker state
    let mut programmer: Option<Box<dyn Programmer>> = None;
    let registry = ChipRegistry::default();

    while let Ok(msg) = rx.recv() {
        match msg {
            GuiMessage::Connect => {
                // First, enumerate devices and send diagnostic info
                match nusb::list_devices() {
                    Ok(devices) => {
                        let wch_devices: Vec<String> = devices
                            .filter(|d| d.vendor_id() == 0x1A86)
                            .map(|d| {
                                let info =
                                    crate::infrastructure::programmer::WchDeviceDatabase::identify(
                                        d.vendor_id(),
                                        d.product_id(),
                                    );
                                format!("{}", info)
                            })
                            .collect();

                        if !wch_devices.is_empty() {
                            tx.send(WorkerMessage::DeviceList(wch_devices)).ok();
                        }
                    }
                    Err(_) => {
                        tx.send(WorkerMessage::Log(
                            "Failed to enumerate USB devices".to_string(),
                        ))
                        .ok();
                    }
                }

                // Now attempt connection
                match crate::infrastructure::programmer::discover(None) {
                    Ok(p) => {
                        let name = p.name().to_string();
                        programmer = Some(p);
                        tx.send(WorkerMessage::Connected(name)).ok();
                    }
                    Err(e) => {
                        tx.send(WorkerMessage::ConnectionFailed(e.to_string())).ok();
                    }
                }
            }
            GuiMessage::DetectChip => {
                if let Some(ref mut p) = programmer {
                    let use_case = DetectChipUseCase::new(registry.clone());
                    match use_case.identify_chip(p.as_mut()) {
                        Ok(spec) => {
                            tx.send(WorkerMessage::ChipDetected(spec)).ok();
                        }
                        Err(e) => {
                            tx.send(WorkerMessage::ChipDetectionFailed(e.to_string()))
                                .ok();
                        }
                    }
                } else {
                    tx.send(WorkerMessage::ConnectionFailed("Not connected".to_string()))
                        .ok();
                }
            }
            GuiMessage::ReadFlash {
                path,
                start,
                length,
            } => {
                if let Some(ref mut p) = programmer {
                    let detect_use_case = DetectChipUseCase::new(registry.clone());
                    let spec = match detect_use_case.identify_chip(p.as_mut()) {
                        Ok(s) => s,
                        Err(e) => {
                            tx.send(WorkerMessage::OperationFailed(format!(
                                "Chip detection failed: {}",
                                e
                            )))
                            .ok();
                            continue;
                        }
                    };

                    let read_len = length.unwrap_or(spec.capacity.as_bytes() - start);
                    let params = ReadParams {
                        address: start,
                        length: read_len,
                        use_ecc: true,
                        ignore_ecc_errors: false,
                        oob_mode: OobMode::None,
                        bad_block_strategy: BadBlockStrategy::Skip,
                        bbt: None,
                        retry_count: 3,
                    };

                    let tx_progress = tx.clone();
                    let result = match spec.flash_type {
                        FlashType::Nand => {
                            let protocol = SpiNand::new(p.as_mut(), spec);
                            let mut use_case = ReadFlashUseCase::new(protocol);
                            use_case.execute(params, |prog| {
                                tx_progress.send(WorkerMessage::Progress(prog)).ok();
                            })
                        }
                        FlashType::Nor => {
                            let protocol = SpiNor::new(p.as_mut(), spec);
                            let mut use_case = ReadFlashUseCase::new(protocol);
                            use_case.execute(params, |prog| {
                                tx_progress.send(WorkerMessage::Progress(prog)).ok();
                            })
                        }
                        FlashType::SpiEeprom => {
                            let protocol = SpiEeprom::new(p.as_mut(), spec);
                            let mut use_case = ReadFlashUseCase::new(protocol);
                            use_case.execute(params, |prog| {
                                tx_progress.send(WorkerMessage::Progress(prog)).ok();
                            })
                        }
                        FlashType::I2cEeprom => {
                            let protocol = I2cEeprom::new(p.as_mut(), spec);
                            let mut use_case = ReadFlashUseCase::new(protocol);
                            use_case.execute(params, |prog| {
                                tx_progress.send(WorkerMessage::Progress(prog)).ok();
                            })
                        }
                        FlashType::MicrowireEeprom => {
                            let protocol = MicrowireEeprom::new(p.as_mut(), spec);
                            let mut use_case = ReadFlashUseCase::new(protocol);
                            use_case.execute(params, |prog| {
                                tx_progress.send(WorkerMessage::Progress(prog)).ok();
                            })
                        }
                    };

                    match result {
                        Ok(data) => {
                            match File::create(&path).and_then(|mut f| f.write_all(&data)) {
                                Ok(_) => {
                                    tx.send(WorkerMessage::DataRead(data)).ok();
                                }
                                Err(e) => {
                                    tx.send(WorkerMessage::OperationFailed(format!(
                                        "File error: {}",
                                        e
                                    )))
                                    .ok();
                                }
                            }
                        }
                        Err(e) => {
                            tx.send(WorkerMessage::OperationFailed(e.to_string())).ok();
                        }
                    }
                } else {
                    tx.send(WorkerMessage::OperationFailed("Not connected".to_string()))
                        .ok();
                }
            }
            GuiMessage::WriteFlash {
                path,
                start,
                verify,
            } => {
                if let Some(ref mut p) = programmer {
                    let detect_use_case = DetectChipUseCase::new(registry.clone());
                    let spec = match detect_use_case.identify_chip(p.as_mut()) {
                        Ok(s) => s,
                        Err(e) => {
                            tx.send(WorkerMessage::OperationFailed(format!(
                                "Chip detection failed: {}",
                                e
                            )))
                            .ok();
                            continue;
                        }
                    };

                    // Read data from file
                    let data = match std::fs::read(&path) {
                        Ok(d) => d,
                        Err(e) => {
                            tx.send(WorkerMessage::OperationFailed(format!(
                                "Failed to read file: {}",
                                e
                            )))
                            .ok();
                            continue;
                        }
                    };

                    let params = WriteParams {
                        address: start,
                        data: &data,
                        use_ecc: true,
                        verify,
                        ignore_ecc_errors: false,
                        oob_mode: OobMode::None,
                        bad_block_strategy: BadBlockStrategy::Skip,
                        bbt: None,
                        retry_count: 3,
                    };

                    let tx_progress = tx.clone();
                    let result = match spec.flash_type {
                        FlashType::Nand => {
                            let protocol = SpiNand::new(p.as_mut(), spec);
                            let mut use_case = WriteFlashUseCase::new(protocol);
                            use_case.execute(params, |prog| {
                                tx_progress.send(WorkerMessage::Progress(prog)).ok();
                            })
                        }
                        FlashType::Nor => {
                            let protocol = SpiNor::new(p.as_mut(), spec);
                            let mut use_case = WriteFlashUseCase::new(protocol);
                            use_case.execute(params, |prog| {
                                tx_progress.send(WorkerMessage::Progress(prog)).ok();
                            })
                        }
                        FlashType::SpiEeprom => {
                            let protocol = SpiEeprom::new(p.as_mut(), spec);
                            let mut use_case = WriteFlashUseCase::new(protocol);
                            use_case.execute(params, |prog| {
                                tx_progress.send(WorkerMessage::Progress(prog)).ok();
                            })
                        }
                        FlashType::I2cEeprom => {
                            let protocol = I2cEeprom::new(p.as_mut(), spec);
                            let mut use_case = WriteFlashUseCase::new(protocol);
                            use_case.execute(params, |prog| {
                                tx_progress.send(WorkerMessage::Progress(prog)).ok();
                            })
                        }
                        FlashType::MicrowireEeprom => {
                            let protocol = MicrowireEeprom::new(p.as_mut(), spec);
                            let mut use_case = WriteFlashUseCase::new(protocol);
                            use_case.execute(params, |prog| {
                                tx_progress.send(WorkerMessage::Progress(prog)).ok();
                            })
                        }
                    };

                    match result {
                        Ok(_) => {
                            tx.send(WorkerMessage::OperationComplete).ok();
                        }
                        Err(e) => {
                            tx.send(WorkerMessage::OperationFailed(e.to_string())).ok();
                        }
                    }
                } else {
                    tx.send(WorkerMessage::OperationFailed("Not connected".to_string()))
                        .ok();
                }
            }
            GuiMessage::EraseFlash { start, length } => {
                if let Some(ref mut p) = programmer {
                    let detect_use_case = DetectChipUseCase::new(registry.clone());
                    let spec = match detect_use_case.identify_chip(p.as_mut()) {
                        Ok(s) => s,
                        Err(e) => {
                            tx.send(WorkerMessage::OperationFailed(format!(
                                "Chip detection failed: {}",
                                e
                            )))
                            .ok();
                            continue;
                        }
                    };

                    let erase_len = length.unwrap_or(spec.capacity.as_bytes() - start);
                    let params = EraseParams {
                        address: start,
                        length: erase_len,
                        bad_block_strategy: BadBlockStrategy::Skip,
                        bbt: None,
                    };

                    let tx_progress = tx.clone();
                    let result = match spec.flash_type {
                        FlashType::Nand => {
                            let protocol = SpiNand::new(p.as_mut(), spec);
                            let mut use_case = EraseFlashUseCase::new(protocol);
                            use_case.execute(params, |prog| {
                                tx_progress.send(WorkerMessage::Progress(prog)).ok();
                            })
                        }
                        FlashType::Nor => {
                            let protocol = SpiNor::new(p.as_mut(), spec);
                            let mut use_case = EraseFlashUseCase::new(protocol);
                            use_case.execute(params, |prog| {
                                tx_progress.send(WorkerMessage::Progress(prog)).ok();
                            })
                        }
                        FlashType::SpiEeprom => {
                            let protocol = SpiEeprom::new(p.as_mut(), spec);
                            let mut use_case = EraseFlashUseCase::new(protocol);
                            use_case.execute(params, |prog| {
                                tx_progress.send(WorkerMessage::Progress(prog)).ok();
                            })
                        }
                        FlashType::I2cEeprom => {
                            let protocol = I2cEeprom::new(p.as_mut(), spec);
                            let mut use_case = EraseFlashUseCase::new(protocol);
                            use_case.execute(params, |prog| {
                                tx_progress.send(WorkerMessage::Progress(prog)).ok();
                            })
                        }
                        FlashType::MicrowireEeprom => {
                            let protocol = MicrowireEeprom::new(p.as_mut(), spec);
                            let mut use_case = EraseFlashUseCase::new(protocol);
                            use_case.execute(params, |prog| {
                                tx_progress.send(WorkerMessage::Progress(prog)).ok();
                            })
                        }
                    };

                    match result {
                        Ok(_) => {
                            tx.send(WorkerMessage::OperationComplete).ok();
                        }
                        Err(e) => {
                            tx.send(WorkerMessage::OperationFailed(e.to_string())).ok();
                        }
                    }
                } else {
                    tx.send(WorkerMessage::OperationFailed("Not connected".to_string()))
                        .ok();
                }
            }
            GuiMessage::Cancel => {
                tx.send(WorkerMessage::Log(
                    "Cancellation ignored (not implemented)".to_string(),
                ))
                .ok();
            }
            GuiMessage::SetSpeed(speed) => {
                if let Some(ref mut p) = programmer {
                    match p.set_speed(speed) {
                        Ok(_) => {
                            tx.send(WorkerMessage::Log(format!("SPI speed set to {}", speed)))
                                .ok();
                        }
                        Err(e) => {
                            tx.send(WorkerMessage::Log(format!("Failed to set speed: {}", e)))
                                .ok();
                        }
                    }
                }
            }
            GuiMessage::SetCsIndex(index) => {
                if let Some(ref mut p) = programmer {
                    match p.select_cs(index) {
                        Ok(_) => {
                            tx.send(WorkerMessage::Log(format!("CS line set to {}", index)))
                                .ok();
                        }
                        Err(e) => {
                            tx.send(WorkerMessage::Log(format!("Failed to set CS: {}", e)))
                                .ok();
                        }
                    }
                }
            }
        }
    }
}
