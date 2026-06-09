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
    let weak_ui = ui.as_weak();
    ui.on_close_menus(move || {
        let ui = weak_ui.unwrap();
        ui.set_file_menu_open(false);
        ui.set_edit_menu_open(false);
        ui.set_help_menu_open(false);
    });

    // --- Открыть файл ---
    let weak_ui = ui.as_weak();
    ui.on_menu_file_open(move || {
        let weak = weak_ui.clone();
        std::thread::spawn(move || {
            let file = rfd::FileDialog::new()
                .add_filter("G-Code", &["txt", "nc", "cnc", "gcode", "ngc"])
                .set_title("Выберите файл G-кода")
                .pick_file();

            if let Some(path) = file {
                let path_str = path.to_string_lossy().to_string();
                let content = fs::read_to_string(&path).ok();

                slint::invoke_from_event_loop(move || {
                    let ui = weak.unwrap();
                    ui.set_file_path(path_str.clone().into());

                    match content {
                        Some(text) => {
                            let lines = text.lines().count();
                            ui.set_code_content(text.into());
                            ui.set_status_text(
                                format!(
                                    "Загружен: {} ({} строк)",
                                    std::path::Path::new(&path_str)
                                        .file_name()
                                        .unwrap()
                                        .to_string_lossy(),
                                    lines
                                )
                                .into(),
                            );
                        }
                        None => {
                            ui.set_status_text(format!("Ошибка чтения файла").into());
                        }
                    }
                })
                .unwrap();
            } else {
                slint::invoke_from_event_loop(move || {
                    let ui = weak.unwrap();
                    ui.set_status_text("Открытие файла отменено.".into());
                })
                .unwrap();
            }
        });
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
    let weak_ui = ui.as_weak();
    ui.on_menu_file_save_as(move || {
        let weak = weak_ui.clone();
        std::thread::spawn(move || {
            let file = rfd::FileDialog::new()
                .add_filter("G-Code", &["nc", "cnc", "txt", "gcode"])
                .set_title("Сохранить как...")
                .save_file();

            if let Some(path) = file {
                let path_str = path.to_string_lossy().to_string();
                // Получаем контент до вызова invoke, чтобы не дергать ui из другого потока

                slint::invoke_from_event_loop(move || {
                    let ui = weak.unwrap();
                    let content = ui.get_code_content().to_string();
                    ui.set_file_path(path_str.clone().into());

                    // fs::write тоже в потоке, но это быстро. Можно и здесь.
                    match std::fs::write(&path_str, &content) {
                        Ok(_) => ui.set_status_text("Сохранено".into()),
                        Err(e) => ui.set_status_text(format!("Ошибка: {}", e).into()),
                    }
                })
                .unwrap();
            }
        });
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

    // --- Редактировать (переключение режима) ---
    let ui_handle = ui.as_weak();
    ui.on_menu_edit_toggle_mode(move || {
        let ui = ui_handle.unwrap();
        let is_edit = ui.get_edit_mode();
        if is_edit {
            ui.set_edit_mode(false);
            ui.set_status_text("Режим просмотра".into());
        } else {
            ui.set_edit_mode(true);
            ui.set_status_text("Режим редактирования. Сохраните изменения.".into());
        }
    });

    // --- Форматировать ---
    let ui_handle = ui.as_weak();
    ui.on_menu_edit_format(move || {
        let ui = ui_handle.unwrap();
        let input = ui.get_code_content().to_string();
        ui.set_status_text("Форматирование...".into());

        match format_code(&input) {
            Ok(formatted) => {
                ui.set_code_content(formatted.into());
                ui.set_status_text("Форматирование завершено".into());
            }
            Err(e) => {
                ui.set_status_text(format!("Ошибка: {:#}", e).into());
            }
        }
    });

    // --- Проверить ошибки ---
    let ui_handle = ui.as_weak();
    ui.on_menu_edit_validate(move || {
        let ui = ui_handle.unwrap();
        let input = ui.get_code_content().to_string();

        match validate_code(&input) {
            Ok(msgs) => {
                if msgs.is_empty() {
                    ui.set_status_text("Ошибок не найдено. Код корректен.".into());
                } else {
                    let has_err = msgs
                        .iter()
                        .any(|e| e.severity == code_parser::shared::Severity::Error);
                    let level = if has_err {
                        "ошибок"
                    } else {
                        "предупреждений"
                    };
                    ui.set_status_text(format!("Найдено {} {}.", msgs.len(), level).into());
                }
            }
            Err(e) => {
                ui.set_status_text(format!("Ошибка: {:#}", e).into());
            }
        }
    });

    // --- О программе ---
    ui.on_menu_help_about(|| {
        println!("G-Code Editor v0.1");
        println!("Форматировщик и валидатор G-кода для станков с ЧПУ.");
        println!("Архитектура: Clean Architecture + Slint UI");
    });

    // --- Горячие клавиши ---
    ui.on_menu_help_shortcuts(|| {
        println!("Горячие клавиши:");
        println!("  F5     — Форматировать");
        println!("  F6     — Проверить ошибки");
        println!("  Ctrl+O — Открыть файл");
        println!("  Ctrl+S — Сохранить");
    });

    ui.run().unwrap();
}

/// Форматирует G-код: лексинг -> парсинг -> валидация -> форматирование.
/// Возвращает отформатированную строку или ошибку anyhow.
fn format_code(input: &str) -> anyhow::Result<String> {
    use anyhow::Context;

    let tokens = code_parser::infrastructure::lexer::tokenize(input);
    let mut parser = code_parser::application::Parser::new(tokens);
    let program = parser.parse_program().context("Ошибка парсинга G-кода")?;

    let errors = code_parser::application::validate(&program);
    if errors
        .iter()
        .any(|e| e.severity == code_parser::shared::Severity::Error)
    {
        anyhow::bail!("Найдено {} ошибок. Форматирование отменено.", errors.len());
    }

    let fmt =
        code_parser::application::Formatter::new(code_parser::application::FormatConfig::default());
    Ok(fmt.format_program(&program))
}

/// Проверяет G-код на ошибки: лексинг -> парсинг -> валидация.
/// Возвращает список сообщений валидации или ошибку anyhow.
fn validate_code(input: &str) -> anyhow::Result<Vec<code_parser::shared::ValidationMessage>> {
    use anyhow::Context;

    let tokens = code_parser::infrastructure::lexer::tokenize(input);
    let mut parser = code_parser::application::Parser::new(tokens);
    let program = parser.parse_program().context("Ошибка парсинга G-кода")?;

    Ok(code_parser::application::validate(&program))
}
