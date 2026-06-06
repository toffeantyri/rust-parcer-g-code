//! Точка входа GUI-приложения (Slint)

use std::fs;

slint::include_modules!();

fn main() {
    let ui = GCodeEditor::new().unwrap();

    let ui_handle = ui.as_weak();
    ui.on_open_file_dialog(move || {
        let ui = ui_handle.unwrap();

        // Открываем системный диалог выбора файла
        let file = rfd::FileDialog::new()
            .add_filter("G-Code", &["txt", "nc", "cnc", "gcode", "ngc"])
            .set_title("Выберите файл G-кода")
            .pick_file();

        if let Some(path) = file {
            let path_str = path.to_string_lossy().to_string();
            ui.set_file_path(path_str.into());

            match fs::read_to_string(&path) {
                Ok(content) => {
                    ui.set_code_content(content.into());
                }
                Err(e) => {
                    ui.set_code_content(format!("Ошибка чтения файла: {}", e).into());
                }
            }
        }
    });

    ui.run().unwrap();
}
