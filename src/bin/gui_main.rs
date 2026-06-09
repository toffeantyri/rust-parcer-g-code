//! Точка входа GUI-приложения (egui)

use code_parser::interfaces::gui::GCodeApp;
use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };
    eframe::run_native(
        "G-Code Editor",
        options,
        Box::new(|_cc| Ok(Box::new(GCodeApp::default()))),
    )
}
