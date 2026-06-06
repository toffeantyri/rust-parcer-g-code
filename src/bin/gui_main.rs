//! Точка входа GUI-приложения (Slint)

slint::include_modules!();

fn main() {
    let ui = GCodeEditor::new().unwrap();
    ui.run().unwrap();
}
