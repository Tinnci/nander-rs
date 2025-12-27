use super::messages::{GuiMessage, WorkerMessage};
use crate::domain::ChipSpec;
use eframe::{egui, App, Frame};
use std::sync::mpsc::{Receiver, Sender};

pub struct NanderApp {
    /// Channel to send commands to the worker thread
    tx: Sender<GuiMessage>,
    /// Channel to receive updates from the worker thread
    rx: Receiver<WorkerMessage>,

    // UI State
    status_text: String,
    programmer_name: Option<String>,
    chip_spec: Option<ChipSpec>,
    is_busy: bool,
    progress: Option<f32>,
    logs: Vec<String>,
    selected_file: Option<std::path::PathBuf>,
    start_address: String,
    length: String,
}

impl NanderApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        tx: Sender<GuiMessage>,
        rx: Receiver<WorkerMessage>,
    ) -> Self {
        // Customize fonts or look here if needed
        Self {
            tx,
            rx,
            status_text: "Ready".to_string(),
            programmer_name: None,
            chip_spec: None,
            is_busy: false,
            progress: None,
            logs: Vec::new(),
            selected_file: None,
            start_address: "0x0".to_string(),
            length: "".to_string(),
        }
    }

    /// Process incoming messages from the worker
    fn handle_messages(&mut self) {
        // Drain all available messages
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                WorkerMessage::Connected(name) => {
                    self.programmer_name = Some(name);
                    self.log("Programmer connected");
                    self.tx.send(GuiMessage::DetectChip).ok(); // Auto-detect on connect
                }
                WorkerMessage::ConnectionFailed(err) => {
                    self.log(&format!("Connection failed: {}", err));
                    self.status_text = format!("Error: {}", err);
                    self.is_busy = false;
                }
                WorkerMessage::ChipDetected(spec) => {
                    self.log(&format!(
                        "Chip detected: {} ({})",
                        spec.name, spec.manufacturer
                    ));
                    self.chip_spec = Some(spec);
                    self.is_busy = false; // Initial detect done
                    self.status_text = "Chip detected".to_string();
                }
                WorkerMessage::ChipDetectionFailed(err) => {
                    self.log(&format!("Chip detection failed: {}", err));
                    self.status_text = "No chip detected".to_string();
                    self.is_busy = false;
                }
                WorkerMessage::Progress(p) => {
                    if p.total > 0 {
                        self.progress = Some(p.current as f32 / p.total as f32);
                        self.status_text = format!(
                            "Working... {:.1}%",
                            (p.current as f64 / p.total as f64) * 100.0
                        );
                    }
                }
                WorkerMessage::OperationComplete => {
                    self.log("Operation completed successfully");
                    self.progress = None;
                    self.is_busy = false;
                    self.status_text = "Ready".to_string();
                }
                WorkerMessage::OperationFailed(err) => {
                    self.log(&format!("Operation failed: {}", err));
                    self.progress = None;
                    self.is_busy = false;
                    self.status_text = "Failed".to_string();
                }
                WorkerMessage::Log(msg) => {
                    self.log(&msg);
                }
            }
        }
    }

    fn log(&mut self, msg: &str) {
        self.logs.push(format!(
            "[{}] {}",
            chrono::Local::now().format("%H:%M:%S"),
            msg
        ));
        // Keep log size manageable
        if self.logs.len() > 1000 {
            self.logs.remove(0);
        }
    }

    fn parse_u32(s: &str) -> Option<u32> {
        let s = s.trim().to_lowercase();
        if let Some(stripped) = s.strip_prefix("0x") {
            u32::from_str_radix(stripped, 16).ok()
        } else {
            s.parse::<u32>().ok()
        }
    }
}

impl App for NanderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Poll for messages every frame
        self.handle_messages();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("nander-rs");

            ui.separator();

            // Status Section
            ui.horizontal(|ui| {
                ui.label("Programmer:");
                if let Some(name) = &self.programmer_name {
                    ui.label(egui::RichText::new(name).color(egui::Color32::GREEN));
                } else {
                    ui.label(egui::RichText::new("Disconnected").color(egui::Color32::RED));
                    if ui.button("Connect").clicked() && !self.is_busy {
                        self.is_busy = true;
                        self.status_text = "Connecting...".to_string();
                        self.tx.send(GuiMessage::Connect).ok();
                    }
                }
            });

            if let Some(spec) = &self.chip_spec {
                ui.group(|ui| {
                    ui.strong("Chip Information");
                    ui.label(format!("Manufacturer: {}", spec.manufacturer));
                    ui.label(format!("Model: {}", spec.name));
                    ui.label(format!("Capacity: {}", spec.capacity));
                    ui.label(format!("Type: {:?}", spec.flash_type));
                });
            }

            ui.separator();

            if self.chip_spec.is_some() {
                ui.group(|ui| {
                    ui.strong("Operations");
                    ui.horizontal(|ui| {
                        if ui.button("Select File...").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                self.selected_file = Some(path);
                            }
                        }
                        if let Some(path) = &self.selected_file {
                            ui.label(format!("File: {}", path.display()));
                        } else {
                            ui.label("No file selected");
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Start Address:");
                        ui.text_edit_singleline(&mut self.start_address);
                        ui.label("Length (bytes):");
                        ui.text_edit_singleline(&mut self.length);
                        if ui.button("Clear").clicked() {
                            self.length.clear();
                        }
                    });

                    ui.horizontal(|ui| {
                        let can_operate = !self.is_busy;
                        let start = Self::parse_u32(&self.start_address).unwrap_or(0);
                        let len = Self::parse_u32(&self.length);

                        if ui
                            .add_enabled(
                                can_operate && self.selected_file.is_some(),
                                egui::Button::new("Read"),
                            )
                            .clicked()
                        {
                            if let Some(path) = &self.selected_file {
                                self.is_busy = true;
                                self.status_text = "Reading...".to_string();
                                self.tx
                                    .send(GuiMessage::ReadFlash {
                                        path: path.clone(),
                                        start,
                                        length: len,
                                    })
                                    .ok();
                            }
                        }

                        if ui
                            .add_enabled(
                                can_operate && self.selected_file.is_some(),
                                egui::Button::new("Write"),
                            )
                            .clicked()
                        {
                            if let Some(path) = &self.selected_file {
                                self.is_busy = true;
                                self.status_text = "Writing...".to_string();
                                self.tx
                                    .send(GuiMessage::WriteFlash {
                                        path: path.clone(),
                                        start,
                                        verify: true,
                                    })
                                    .ok();
                            }
                        }

                        if ui
                            .add_enabled(can_operate, egui::Button::new("Erase"))
                            .clicked()
                        {
                            self.is_busy = true;
                            self.status_text = "Erasing...".to_string();
                            self.tx
                                .send(GuiMessage::EraseFlash { start, length: len })
                                .ok();
                        }
                    });
                });
            }

            ui.separator();

            // Progress Bar
            if let Some(prog) = self.progress {
                ui.add(egui::ProgressBar::new(prog).show_percentage());
            }

            // Status Bar
            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.label(&self.status_text);
            });

            ui.separator();

            // Log View
            ui.collapsing("Logs", |ui| {
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for log in &self.logs {
                            ui.monospace(log);
                        }
                    });
            });
        });

        // If working, request constant repaints to update progress smoothly
        if self.is_busy {
            ctx.request_repaint();
        }
    }
}
