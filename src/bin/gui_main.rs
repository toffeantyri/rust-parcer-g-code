//! Точка входа GUI-приложения (egui)

use code_parser::interfaces::gui::GCodeApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "G-Code Editor",
        options,
        Box::new(|_cc| Ok(Box::new(GCodeApp::default()))),
    )
}
