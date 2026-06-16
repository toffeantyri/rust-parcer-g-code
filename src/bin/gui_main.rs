//! Точка входа GUI-приложения (egui)

use code_parser::data_layer::spawn_data_layer;
use code_parser::interfaces::gui::GCodeApp;
use egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };

    // Запускаем data layer в отдельном потоке
    let (cmd_tx, evt_rx) = spawn_data_layer();

    eframe::run_native(
        "G-Code Editor",
        options,
        Box::new(move |_cc| Ok(Box::new(GCodeApp::new(cmd_tx, evt_rx)))),
    )
}
