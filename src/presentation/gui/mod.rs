pub mod app;
pub mod messages;
pub mod worker;

use eframe::{run_native, NativeOptions};
use std::sync::mpsc::channel;
use std::thread;

pub fn run() -> eframe::Result<()> {
    // Create channels for communication
    let (tx_gui, rx_gui) = channel(); // Worker -> GUI
    let (tx_worker, rx_worker) = channel(); // GUI -> Worker

    // Spawn the background worker thread
    thread::spawn(move || {
        worker::run_worker(rx_worker, tx_gui);
    });

    let native_options = NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([900.0, 650.0])
            .with_min_inner_size([600.0, 450.0])
            .with_title("nander-rs"),
        ..Default::default()
    };

    run_native(
        "nander-rs",
        native_options,
        Box::new(|cc| Ok(Box::new(app::NanderApp::new(cc, tx_worker, rx_gui)))),
    )
}
