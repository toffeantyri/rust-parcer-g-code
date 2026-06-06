//! Точка входа GUI-приложения (Slint)

use std::fs;

slint::include_modules!();

fn main() {
    let ui = GCodeEditor::new().unwrap();

    ui.set_status_text("Готов к работе. Откройте файл G-кода.".into());

    // --- Управление меню ---
    let weak_ui = ui.as_weak();
    ui.on_menu_file_toggle(move || {
        let ui = weak_ui.unwrap();
        let was_open = ui.get_file_menu_open();
        ui.set_edit_menu_open(false);
        ui.set_help_menu_open(false);
        ui.set_file_menu_open(!was_open);
    });
    let weak_ui = ui.as_weak();
    ui.on_menu_edit_toggle(move || {
        let ui = weak_ui.unwrap();
        ui.set_file_menu_open(false);
        ui.set_help_menu_open(false);
        let was_open = ui.get_edit_menu_open();
        ui.set_edit_menu_open(!was_open);
    });
    let weak_ui = ui.as_weak();
    ui.on_menu_help_toggle(move || {
        let ui = weak_ui.unwrap();
        ui.set_file_menu_open(false);
        ui.set_edit_menu_open(false);
        let was_open = ui.get_help_menu_open();
        ui.set_help_menu_open(!was_open);
    });
    ui.on_close_menus(|| {});

    // --- Открыть файл ---
    let ui_handle = ui.as_weak();
    ui.on_menu_file_open(move || {
        let ui = ui_handle.unwrap();
        ui.set_status_text("Выберите файл...".into());

        let file = rfd::FileDialog::new()
            .add_filter("G-Code", &["txt", "nc", "cnc", "gcode", "ngc"])
            .set_title("Выберите файл G-кода")
            .pick_file();

        if let Some(path) = file {
            let path_str = path.to_string_lossy().to_string();
            ui.set_file_path(path_str.into());

            match fs::read_to_string(&path) {
                Ok(content) => {
                    let lines = content.lines().count();
                    ui.set_code_content(content.into());
                    ui.set_status_text(
                        format!(
                            "Загружен: {} ({} строк)",
                            path.file_name().unwrap().to_string_lossy(),
                            lines
                        )
                        .into(),
                    );
                }
                Err(e) => {
                    ui.set_code_content(format!("Ошибка: {}", e).into());
                    ui.set_status_text(format!("Ошибка: {}", e).into());
                }
            }
        } else {
            ui.set_status_text("Открытие файла отменено.".into());
        }
    });

    // --- Сохранить ---
    let ui_handle = ui.as_weak();
    ui.on_menu_file_save(move || {
        let ui = ui_handle.unwrap();
        let path = ui.get_file_path();

        if path.is_empty() {
            ui.invoke_menu_file_save_as();
            return;
        }

        let content = ui.get_code_content().to_string();
        match fs::write(path.as_str(), &content) {
            Ok(_) => ui.set_status_text("Сохранено".into()),
            Err(e) => ui.set_status_text(format!("Ошибка сохранения: {}", e).into()),
        }
    });

    // --- Сохранить как ---
    let ui_handle = ui.as_weak();
    ui.on_menu_file_save_as(move || {
        let ui = ui_handle.unwrap();
        let file = rfd::FileDialog::new()
            .add_filter("G-Code", &["nc", "cnc", "txt", "gcode"])
            .set_title("Сохранить как...")
            .save_file();

        if let Some(path) = file {
            let path_str = path.to_string_lossy().to_string();
            let content = ui.get_code_content().to_string();
            match fs::write(&path, &content) {
                Ok(_) => {
                    ui.set_file_path(path_str.into());
                    ui.set_status_text("Сохранено".into());
                }
                Err(e) => ui.set_status_text(format!("Ошибка: {}", e).into()),
            }
        }
    });

    // --- Закрыть файл ---
    let ui_handle = ui.as_weak();
    ui.on_menu_file_close(move || {
        let ui = ui_handle.unwrap();
        ui.set_file_path(slint::SharedString::default());
        ui.set_code_content(slint::SharedString::default());
        ui.set_status_text("Файл закрыт.".into());
    });

    // --- Выход ---
    ui.on_menu_file_exit(move || {
        std::process::exit(0);
    });

    // --- Форматировать ---
    let ui_handle = ui.as_weak();
    ui.on_menu_edit_format(move || {
        let ui = ui_handle.unwrap();
        let input = ui.get_code_content().to_string();
        ui.set_status_text("Форматирование...".into());

        let tokens = code_parser::infrastructure::lexer::tokenize(&input);
        let mut parser = code_parser::application::Parser::new(tokens);
        let program = match parser.parse_program() {
            Ok(p) => p,
            Err(e) => {
                ui.set_status_text(format!("Ошибка парсинга: {}", e).into());
                return;
            }
        };

        let errors = code_parser::application::validate(&program);
        if errors
            .iter()
            .any(|e| e.severity == code_parser::shared::Severity::Error)
        {
            ui.set_status_text(
                format!("Найдено {} ошибок. Форматирование отменено.", errors.len()).into(),
            );
            return;
        }

        let fmt = code_parser::application::Formatter::new(
            code_parser::application::FormatConfig::default(),
        );
        ui.set_code_content(fmt.format_program(&program).into());
        ui.set_status_text("Форматирование завершено".into());
    });

    // --- Проверить ошибки ---
    let ui_handle = ui.as_weak();
    ui.on_menu_edit_validate(move || {
        let ui = ui_handle.unwrap();
        let input = ui.get_code_content().to_string();

        let tokens = code_parser::infrastructure::lexer::tokenize(&input);
        let mut parser = code_parser::application::Parser::new(tokens);
        let program = match parser.parse_program() {
            Ok(p) => p,
            Err(e) => {
                ui.set_status_text(format!("Ошибка парсинга: {}", e).into());
                return;
            }
        };

        let errors = code_parser::application::validate(&program);
        if errors.is_empty() {
            ui.set_status_text("Ошибок не найдено. Код корректен.".into());
        } else {
            let has_err = errors
                .iter()
                .any(|e| e.severity == code_parser::shared::Severity::Error);
            let level = if has_err {
                "ошибок"
            } else {
                "предупреждений"
            };
            ui.set_status_text(format!("Найдено {} {}.", errors.len(), level).into());
        }
    });

    // --- О программе ---
    ui.on_menu_help_about(|| {
        eprintln!("G-Code Editor v0.1");
        eprintln!("Форматировщик и валидатор G-кода для станков с ЧПУ.");
        eprintln!("Архитектура: Clean Architecture + Slint UI");
    });

    // --- Горячие клавиши ---
    ui.on_menu_help_shortcuts(|| {
        eprintln!("Горячие клавиши:");
        eprintln!("  F5     — Форматировать");
        eprintln!("  F6     — Проверить ошибки");
        eprintln!("  Ctrl+O — Открыть файл");
        eprintln!("  Ctrl+S — Сохранить");
    });

    ui.run().unwrap();
}
