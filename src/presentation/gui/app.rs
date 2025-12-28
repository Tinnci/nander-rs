use super::messages::{GuiMessage, WorkerMessage};
use crate::domain::serial_analysis::RollingQualityAnalyzer;
use crate::domain::ChipSpec;
use crate::infrastructure::programmer::traits::{Parity, SerialConfig, StopBits};
use eframe::{egui, App, Frame};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, Sender};

#[derive(PartialEq, Serialize, Deserialize)]
enum Tab {
    Read,
    Write,
    Erase,
    Console,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct NanderApp {
    /// Channel to send commands to the worker thread
    #[serde(skip)]
    tx: Sender<GuiMessage>,
    /// Channel to receive updates from the worker thread
    #[serde(skip)]
    rx: Receiver<WorkerMessage>,

    // UI State
    active_tab: Tab,
    spi_speed: u8,
    cs_index: u8,
    #[serde(skip)]
    status_text: String,
    #[serde(skip)]
    programmer_name: Option<String>,
    #[serde(skip)]
    chip_spec: Option<ChipSpec>,
    #[serde(skip)]
    is_busy: bool,
    #[serde(skip)]
    progress: Option<f32>,
    #[serde(skip)]
    logs: Vec<String>,
    logs_open: bool,
    selected_file: Option<std::path::PathBuf>,
    start_address: String,
    length: String,
    #[serde(skip)]
    preview_data: Vec<u8>,

    // =========================================================================
    // Console/Serial State
    // =========================================================================
    #[serde(skip)]
    serial_connected: bool,
    #[serde(skip)]
    serial_port_name: Option<String>,
    /// TX input buffer
    console_tx_input: String,
    /// RX display buffer
    #[serde(skip)]
    console_rx_buffer: Vec<u8>,
    /// Display mode: true = Hex, false = ASCII
    console_hex_mode: bool,
    /// Auto-scroll RX area
    console_auto_scroll: bool,
    /// Add timestamp to received data
    console_timestamp: bool,
    /// Send with newline
    console_send_newline: bool,
    /// Newline type: 0=LF, 1=CR, 2=CRLF
    console_newline_type: u8,
    /// Baud rate
    console_baud_rate: u32,
    /// Data bits
    console_data_bits: u8,
    /// Parity
    #[serde(skip)]
    console_parity: Parity,
    /// Stop bits
    #[serde(skip)]
    console_stop_bits: StopBits,
    /// DTR state
    #[serde(skip)]
    console_dtr: bool,
    /// RTS state
    #[serde(skip)]
    console_rts: bool,
    /// TX byte counter
    #[serde(skip)]
    console_tx_count: usize,
    /// RX byte counter
    #[serde(skip)]
    console_rx_count: usize,
    /// Data quality analyzer
    #[serde(skip)]
    console_quality: RollingQualityAnalyzer,
    /// Baud rate detection results (baud, confidence, preview, protocol)
    #[serde(skip)]
    console_baud_results: Vec<(u32, f32, String, String)>,
}

impl Default for NanderApp {
    fn default() -> Self {
        let (tx, _) = std::sync::mpsc::channel();
        let (_, rx) = std::sync::mpsc::channel();
        Self {
            tx,
            rx,
            active_tab: Tab::Read,
            spi_speed: 5,
            cs_index: 0,
            status_text: "Ready".to_string(),
            programmer_name: None,
            chip_spec: None,
            is_busy: false,
            progress: None,
            logs: Vec::new(),
            logs_open: false,
            selected_file: None,
            start_address: "0x0".to_string(),
            length: "".to_string(),
            preview_data: Vec::new(),
            // Console defaults
            serial_connected: false,
            serial_port_name: None,
            console_tx_input: String::new(),
            console_rx_buffer: Vec::new(),
            console_hex_mode: false,
            console_auto_scroll: true,
            console_timestamp: false,
            console_send_newline: true,
            console_newline_type: 0, // LF
            console_baud_rate: 115200,
            console_data_bits: 8,
            console_parity: Parity::None,
            console_stop_bits: StopBits::One,
            console_dtr: false,
            console_rts: false,
            console_tx_count: 0,
            console_rx_count: 0,
            console_quality: RollingQualityAnalyzer::default(),
            console_baud_results: Vec::new(),
        }
    }
}

impl NanderApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        tx: Sender<GuiMessage>,
        rx: Receiver<WorkerMessage>,
    ) -> Self {
        let mut app = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Self::default()
        };

        app.tx = tx;
        app.rx = rx;
        app.status_text = "Ready".to_string();
        app.is_busy = false;
        app.progress = None;
        app.logs = Vec::new();
        app.preview_data = Vec::new();

        app
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
                    self.status_text = "Connection Failed (see logs)".to_string();
                    self.is_busy = false;
                    self.logs_open = true; // Open logs on error
                }
                WorkerMessage::Disconnected => {
                    self.programmer_name = None;
                    self.chip_spec = None;
                    self.log("Programmer disconnected");
                    self.status_text = "Programmer Disconnected".to_string();
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
                    self.status_text = "Chip Detection Failed (see logs)".to_string();
                    self.is_busy = false;
                    self.logs_open = true;
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
                WorkerMessage::DataRead(data) => {
                    self.log(&format!("Read {} bytes successfully", data.len()));
                    self.preview_data = data;
                    self.progress = None;
                    self.is_busy = false;
                    self.status_text = "Ready".to_string();
                }
                WorkerMessage::OperationFailed(err) => {
                    self.log(&format!("Operation failed: {}", err));
                    self.progress = None;
                    self.is_busy = false;
                    self.status_text = "Operation Failed (see logs)".to_string();
                    self.logs_open = true; // Open logs on error
                }
                WorkerMessage::Log(msg) => {
                    self.log(&msg);
                }
                WorkerMessage::DeviceList(devices) => {
                    self.log("=== Detected WCH Devices ===");
                    for device in devices {
                        self.log(&format!("  {}", device));
                    }
                    self.log("===========================");
                }
                // Serial/Console messages
                WorkerMessage::SerialConnected(name) => {
                    self.serial_connected = true;
                    self.serial_port_name = Some(name.clone());
                    self.log(&format!("Serial port connected: {}", name));
                }
                WorkerMessage::SerialDisconnected => {
                    self.serial_connected = false;
                    self.serial_port_name = None;
                    self.log("Serial port disconnected");
                }
                WorkerMessage::SerialConnectionFailed(err) => {
                    self.log(&format!("Serial connection failed: {}", err));
                    self.logs_open = true;
                }
                WorkerMessage::SerialDataReceived(data) => {
                    self.console_rx_count += data.len();
                    self.console_quality.process(&data);
                    if self.console_timestamp {
                        let timestamp = chrono::Local::now().format("%H:%M:%S.%3f ");
                        self.console_rx_buffer
                            .extend_from_slice(timestamp.to_string().as_bytes());
                    }
                    self.console_rx_buffer.extend_from_slice(&data);
                    // Limit buffer size (keep last 64KB)
                    if self.console_rx_buffer.len() > 64 * 1024 {
                        let start = self.console_rx_buffer.len() - 64 * 1024;
                        self.console_rx_buffer = self.console_rx_buffer[start..].to_vec();
                    }
                }
                WorkerMessage::SerialSendComplete(bytes) => {
                    self.console_tx_count += bytes;
                }
                WorkerMessage::SerialAutoDetectProgress(p) => {
                    self.progress = Some(p);
                    self.status_text = format!("Scanning baud rates... {:.0}%", p * 100.0);
                }
                WorkerMessage::SerialBaudDetectionResults(results) => {
                    self.progress = None;
                    self.is_busy = false;
                    self.status_text = "Detection complete".to_string();
                    self.console_baud_results = results.clone();
                    self.log("=== Baud Rate Detection Results ===");
                    for (baud, confidence, preview, protocol) in results {
                        self.log(&format!(
                            "  {} baud: {:.1}% confidence | Protocol: {} | Preview: \"{}\"",
                            baud,
                            confidence * 100.0,
                            protocol,
                            preview.escape_debug()
                        ));
                    }
                    self.log("===================================");
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

    fn render_hex_view(&mut self, ui: &mut egui::Ui) {
        if self.preview_data.is_empty() {
            ui.label("No data to display. Read flash to see content.");
            return;
        }

        ui.monospace("Offset    00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F  ASCII");
        ui.separator();

        let bytes_per_row = 16;
        let total_rows = self.preview_data.len().div_ceil(bytes_per_row);

        egui::ScrollArea::vertical()
            .max_height(400.0)
            .auto_shrink([false, false])
            .show_rows(ui, 14.0, total_rows, |ui, row_range| {
                for r in row_range {
                    let offset = r * bytes_per_row;
                    if offset >= self.preview_data.len() {
                        break;
                    }
                    let end = std::cmp::min(offset + bytes_per_row, self.preview_data.len());
                    let row_data = &self.preview_data[offset..end];

                    let mut hex_str = String::with_capacity(50);
                    let mut ascii_str = String::with_capacity(bytes_per_row);

                    for (i, &b) in row_data.iter().enumerate() {
                        hex_str.push_str(&format!("{:02X} ", b));
                        if i == 7 {
                            hex_str.push(' ');
                        }

                        if (32..=126).contains(&b) {
                            ascii_str.push(b as char);
                        } else {
                            ascii_str.push('.');
                        }
                    }

                    // Align columns
                    while hex_str.len() < 49 {
                        hex_str.push(' ');
                    }

                    ui.monospace(format!("{:08X}  {}  {}", offset, hex_str, ascii_str));
                }
            });
    }

    fn load_file_preview(&mut self) {
        if let Some(path) = &self.selected_file {
            use std::io::Read;
            if let Ok(mut f) = std::fs::File::open(path) {
                let mut buffer = vec![0u8; 64 * 1024]; // 64KB limit for preview
                if let Ok(n) = f.read(&mut buffer) {
                    buffer.truncate(n);
                    self.preview_data = buffer;
                    self.log(&format!("Loaded file preview (first {} bytes)", n));
                }
            }
        }
    }

    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                if let Some(file) = i.raw.dropped_files.first() {
                    if let Some(path) = &file.path {
                        self.selected_file = Some(path.clone());
                        self.load_file_preview();
                    }
                }
            }
        });
    }

    fn parse_u32(s: &str) -> Option<u32> {
        let s = s.trim().to_lowercase();
        if let Some(stripped) = s.strip_prefix("0x") {
            u32::from_str_radix(stripped, 16).ok()
        } else {
            s.parse::<u32>().ok()
        }
    }

    // =========================================================================
    // Console Tab Rendering
    // =========================================================================

    fn render_console_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔌 Serial Console");

        // Top row: Connection status and controls
        ui.horizontal(|ui| {
            // Connection status
            if self.serial_connected {
                if let Some(name) = &self.serial_port_name {
                    ui.label(
                        egui::RichText::new(format!("✓ {}", name)).color(egui::Color32::GREEN),
                    );
                }
                if ui.button("Disconnect").clicked() {
                    self.tx.send(GuiMessage::SerialDisconnect).ok();
                }
            } else {
                ui.label(egui::RichText::new("○ Not Connected").color(egui::Color32::GRAY));
                if ui.button("Connect").clicked() {
                    // Apply config and connect
                    let config = SerialConfig {
                        baud_rate: self.console_baud_rate,
                        data_bits: self.console_data_bits,
                        parity: self.console_parity,
                        stop_bits: self.console_stop_bits,
                    };
                    self.tx.send(GuiMessage::SerialConfigure(config)).ok();
                    self.tx.send(GuiMessage::SerialConnect).ok();
                }
            }

            ui.separator();

            ui.label(format!("TX: {} bytes", self.console_tx_count));
            ui.label(format!("RX: {} bytes", self.console_rx_count));

            if self.serial_connected {
                let quality = self.console_quality.quality_level();
                let score = self.console_quality.quality_score();
                let (r, g, b) = quality.color();

                ui.separator();
                ui.label("Quality:");
                ui.add(
                    egui::ProgressBar::new(score)
                        .text(quality.description())
                        .fill(egui::Color32::from_rgb(r, g, b)),
                );

                let protocol = self.console_quality.protocol();
                if protocol != crate::domain::serial_analysis::ProtocolType::Unknown {
                    ui.label(format!("Protocol: {}", protocol.description()));
                }
            }

            if ui.button("Reset").clicked() {
                self.console_tx_count = 0;
                self.console_rx_count = 0;
                self.console_quality.reset();
            }
        });

        ui.separator();

        // Two-column layout: Config on left, Console on right
        ui.columns(2, |columns| {
            // Left column: Configuration
            columns[0].group(|ui| {
                ui.heading("Configuration");

                // Baud Rate
                ui.horizontal(|ui| {
                    ui.label("Baud Rate:");
                    egui::ComboBox::from_id_salt("baud_rate")
                        .selected_text(format!("{}", self.console_baud_rate))
                        .show_ui(ui, |ui| {
                            for &rate in SerialConfig::common_baud_rates() {
                                ui.selectable_value(
                                    &mut self.console_baud_rate,
                                    rate,
                                    format!("{}", rate),
                                );
                            }
                        });

                    if ui
                        .button("🔍 Auto")
                        .on_hover_text("Auto-detect baud rate (scans common rates)")
                        .clicked()
                    {
                        self.tx.send(GuiMessage::SerialAutoDetectBaud).ok();
                        self.is_busy = true;
                    }
                });

                // Display scan results
                if !self.console_baud_results.is_empty() {
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Scan Results:").strong());
                        for (baud, confidence, preview, protocol) in
                            self.console_baud_results.clone()
                        {
                            let mut text = format!("{} baud ({:.0}%)", baud, confidence * 100.0);
                            if protocol != "Unknown Protocol" {
                                text = format!("{} - {}", text, protocol);
                            }

                            let color = if confidence > 0.8 {
                                egui::Color32::GREEN
                            } else if confidence > 0.4 {
                                egui::Color32::YELLOW
                            } else {
                                egui::Color32::GRAY
                            };

                            if ui
                                .button(egui::RichText::new(text).color(color))
                                .on_hover_text(format!(
                                    "Preview: \"{}\"",
                                    preview.trim().escape_debug()
                                ))
                                .clicked()
                            {
                                self.console_baud_rate = baud;
                                let config = SerialConfig {
                                    baud_rate: baud,
                                    data_bits: self.console_data_bits,
                                    parity: self.console_parity,
                                    stop_bits: self.console_stop_bits,
                                };
                                self.tx.send(GuiMessage::SerialConfigure(config)).ok();
                                self.console_baud_results.clear();
                            }
                        }
                        if ui.button("Clear Results").clicked() {
                            self.console_baud_results.clear();
                        }
                    });
                }

                // Data Bits
                ui.horizontal(|ui| {
                    ui.label("Data Bits:");
                    egui::ComboBox::from_id_salt("data_bits")
                        .selected_text(format!("{}", self.console_data_bits))
                        .show_ui(ui, |ui| {
                            for bits in [5u8, 6, 7, 8] {
                                ui.selectable_value(
                                    &mut self.console_data_bits,
                                    bits,
                                    format!("{}", bits),
                                );
                            }
                        });
                });

                // Parity
                ui.horizontal(|ui| {
                    ui.label("Parity:");
                    egui::ComboBox::from_id_salt("parity")
                        .selected_text(self.console_parity.as_str())
                        .show_ui(ui, |ui| {
                            for &p in Parity::all() {
                                ui.selectable_value(&mut self.console_parity, p, p.as_str());
                            }
                        });
                });

                // Stop Bits
                ui.horizontal(|ui| {
                    ui.label("Stop Bits:");
                    egui::ComboBox::from_id_salt("stop_bits")
                        .selected_text(self.console_stop_bits.as_str())
                        .show_ui(ui, |ui| {
                            for &s in StopBits::all() {
                                ui.selectable_value(&mut self.console_stop_bits, s, s.as_str());
                            }
                        });
                });

                ui.separator();

                // Flow Control (DTR/RTS)
                ui.heading("Flow Control");
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.console_dtr, "DTR").changed() {
                        self.tx
                            .send(GuiMessage::SerialSetDtr(self.console_dtr))
                            .ok();
                    }
                    if ui.checkbox(&mut self.console_rts, "RTS").changed() {
                        self.tx
                            .send(GuiMessage::SerialSetRts(self.console_rts))
                            .ok();
                    }
                });

                ui.separator();

                // Display Options
                ui.heading("Display Options");
                ui.checkbox(&mut self.console_hex_mode, "Hex Mode");
                ui.checkbox(&mut self.console_auto_scroll, "Auto-scroll");
                ui.checkbox(&mut self.console_timestamp, "Show Timestamp");

                ui.separator();

                // Send Options
                ui.heading("Send Options");
                ui.checkbox(&mut self.console_send_newline, "Append Newline");
                if self.console_send_newline {
                    ui.horizontal(|ui| {
                        ui.label("Newline:");
                        ui.selectable_value(&mut self.console_newline_type, 0, "LF");
                        ui.selectable_value(&mut self.console_newline_type, 1, "CR");
                        ui.selectable_value(&mut self.console_newline_type, 2, "CRLF");
                    });
                }

                ui.separator();

                // Clear buffers
                if ui.button("Clear RX Buffer").clicked() {
                    self.console_rx_buffer.clear();
                }
            });

            // Right column: TX/RX Console
            columns[1].group(|ui| {
                ui.heading("Received Data");

                // RX Display Area
                let rx_height = ui.available_height() * 0.6;
                egui::ScrollArea::vertical()
                    .id_salt("rx_scroll")
                    .max_height(rx_height)
                    .stick_to_bottom(self.console_auto_scroll)
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        if self.console_hex_mode {
                            // Hex display
                            self.render_console_hex_view(ui, &self.console_rx_buffer.clone());
                        } else {
                            // ASCII display
                            let text = String::from_utf8_lossy(&self.console_rx_buffer);
                            ui.add(
                                egui::TextEdit::multiline(&mut text.to_string())
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY)
                                    .interactive(false),
                            );
                        }
                    });

                ui.separator();

                ui.heading("Send Data");

                // TX Input Area
                ui.horizontal(|ui| {
                    let response = ui.add(
                        egui::TextEdit::multiline(&mut self.console_tx_input)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(ui.available_width() - 80.0)
                            .desired_rows(3),
                    );

                    ui.vertical(|ui| {
                        let can_send = self.serial_connected && !self.console_tx_input.is_empty();
                        if ui
                            .add_enabled(can_send, egui::Button::new("Send"))
                            .clicked()
                            || (response.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                && ui.input(|i| i.modifiers.ctrl))
                        {
                            self.send_console_data();
                        }

                        if ui.button("Clear").clicked() {
                            self.console_tx_input.clear();
                        }
                    });
                });

                ui.label(
                    egui::RichText::new("Tip: Ctrl+Enter to send")
                        .small()
                        .color(egui::Color32::GRAY),
                );
            });
        });
    }

    fn render_console_hex_view(&self, ui: &mut egui::Ui, data: &[u8]) {
        if data.is_empty() {
            ui.label("No data received yet.");
            return;
        }

        let bytes_per_row = 16;
        let total_rows = data.len().div_ceil(bytes_per_row);

        for r in 0..total_rows.min(100) {
            // Limit display to 100 rows for performance
            let offset = r * bytes_per_row;
            if offset >= data.len() {
                break;
            }
            let end = std::cmp::min(offset + bytes_per_row, data.len());
            let row_data = &data[offset..end];

            let mut hex_str = String::with_capacity(50);
            let mut ascii_str = String::with_capacity(bytes_per_row);

            for (i, &b) in row_data.iter().enumerate() {
                hex_str.push_str(&format!("{:02X} ", b));
                if i == 7 {
                    hex_str.push(' ');
                }
                if (32..=126).contains(&b) {
                    ascii_str.push(b as char);
                } else {
                    ascii_str.push('.');
                }
            }

            while hex_str.len() < 49 {
                hex_str.push(' ');
            }

            ui.monospace(format!("{:04X}  {}  {}", offset, hex_str, ascii_str));
        }

        if total_rows > 100 {
            ui.label(format!("... and {} more rows", total_rows - 100));
        }
    }

    fn send_console_data(&mut self) {
        if self.console_tx_input.is_empty() {
            return;
        }

        let mut data = self.console_tx_input.as_bytes().to_vec();

        // Append newline if configured
        if self.console_send_newline {
            match self.console_newline_type {
                0 => data.push(b'\n'),                // LF
                1 => data.push(b'\r'),                // CR
                2 => data.extend_from_slice(b"\r\n"), // CRLF
                _ => {}
            }
        }

        self.tx.send(GuiMessage::SerialSend(data)).ok();
        self.console_tx_input.clear();
    }
}

impl App for NanderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Poll for messages every frame
        self.handle_messages();

        // 1. Top Panel (Menu)
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });

        // 2. Left Side Panel (Settings)
        egui::SidePanel::left("settings_panel")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Settings");
                ui.separator();

                ui.vertical(|ui| {
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

                    ui.separator();

                    ui.label("SPI Speed:");
                    if ui
                        .add(egui::Slider::new(&mut self.spi_speed, 0..=7).text("Level"))
                        .changed()
                    {
                        self.tx.send(GuiMessage::SetSpeed(self.spi_speed)).ok();
                    }

                    ui.separator();

                    ui.label("Chip Select (CS):");
                    ui.horizontal(|ui| {
                        if ui.selectable_value(&mut self.cs_index, 0, "CS0").clicked() {
                            self.tx.send(GuiMessage::SetCsIndex(0)).ok();
                        }
                        if ui.selectable_value(&mut self.cs_index, 1, "CS1").clicked() {
                            self.tx.send(GuiMessage::SetCsIndex(1)).ok();
                        }
                    });

                    ui.separator();

                    if ui
                        .add_enabled(!self.is_busy, egui::Button::new("Detect Chip"))
                        .clicked()
                    {
                        self.is_busy = true;
                        self.tx.send(GuiMessage::DetectChip).ok();
                    }

                    if let Some(spec) = &self.chip_spec {
                        ui.group(|ui| {
                            ui.strong("Chip Info");
                            ui.label(format!("Name: {}", spec.name));
                            ui.label(format!("Size: {}", spec.capacity));
                            ui.label(format!("Type: {:?}", spec.flash_type));
                        });
                    }
                });
            });

        // 3. Bottom Panel (Status & Logs)
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.strong("Status:");
                if let Some(prog) = self.progress {
                    ui.add(
                        egui::ProgressBar::new(prog)
                            .show_percentage()
                            .desired_width(200.0),
                    );
                }
            });
            ui.label(&self.status_text);

            ui.separator();

            let collapsing = egui::CollapsingHeader::new("Logs").open(Some(self.logs_open));
            let resp = collapsing.show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .max_height(100.0)
                    .show(ui, |ui| {
                        for log in &self.logs {
                            ui.monospace(log);
                        }
                    });
            });

            if resp.header_response.clicked() {
                self.logs_open = !self.logs_open;
            }
        });

        // 4. Central Panel (Operations)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, Tab::Read, "Read");
                ui.selectable_value(&mut self.active_tab, Tab::Write, "Write");
                ui.selectable_value(&mut self.active_tab, Tab::Erase, "Erase");
                ui.separator();
                ui.selectable_value(&mut self.active_tab, Tab::Console, "🔌 Console");
            });

            ui.separator();

            let can_operate = !self.is_busy && self.programmer_name.is_some();
            let start = Self::parse_u32(&self.start_address).unwrap_or(0);
            let len = Self::parse_u32(&self.length);

            match self.active_tab {
                Tab::Read => {
                    ui.heading("Read Flash");
                    ui.horizontal(|ui| {
                        if ui.button("Select Save Path...").clicked() {
                            if let Some(path) = rfd::FileDialog::new().save_file() {
                                self.selected_file = Some(path);
                            }
                        }
                        if let Some(path) = &self.selected_file {
                            ui.label(format!("Path: {}", path.display()));
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Start Address:");
                        ui.text_edit_singleline(&mut self.start_address);
                        ui.label("Length:");
                        ui.text_edit_singleline(&mut self.length);
                    });

                    if ui
                        .add_enabled(
                            can_operate && self.selected_file.is_some(),
                            egui::Button::new("Start Reading"),
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

                    ui.separator();
                    ui.label("Hex Preview:");
                    self.render_hex_view(ui);
                }
                Tab::Write => {
                    ui.heading("Write Flash");
                    ui.horizontal(|ui| {
                        if ui.button("Select File to Write...").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                self.selected_file = Some(path);
                                self.load_file_preview();
                            }
                        }
                        if let Some(path) = &self.selected_file {
                            ui.label(format!("File: {}", path.display()));
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Start Address:");
                        ui.text_edit_singleline(&mut self.start_address);
                    });

                    if ui
                        .add_enabled(
                            can_operate && self.selected_file.is_some(),
                            egui::Button::new("Start Writing"),
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

                    ui.separator();
                    ui.label("File Preview:");
                    self.render_hex_view(ui);
                }
                Tab::Erase => {
                    ui.heading("Erase Flash");
                    ui.horizontal(|ui| {
                        ui.label("Start Address:");
                        ui.text_edit_singleline(&mut self.start_address);
                        ui.label("Length:");
                        ui.text_edit_singleline(&mut self.length);
                    });

                    if ui
                        .add_enabled(can_operate, egui::Button::new("Start Erasing"))
                        .clicked()
                    {
                        self.is_busy = true;
                        self.status_text = "Erasing...".to_string();
                        self.tx
                            .send(GuiMessage::EraseFlash { start, length: len })
                            .ok();
                    }
                }
                Tab::Console => {
                    self.render_console_tab(ui);
                }
            }
        });

        // Handle File Drops
        self.handle_dropped_files(ctx);

        // If working, request constant repaints to update progress smoothly
        if self.is_busy {
            ctx.request_repaint();
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
