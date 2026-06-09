//! Update — редьюсер: применяет намерения к модели

use super::intent::Intent;
use super::model::Model;
use crate::shared::Severity;

impl Model {
    /// Применяет намерение пользователя к модели.
    /// Вызывается из App::update() для каждого Intent.
    pub fn apply(&mut self, intent: &Intent) {
        match intent {
            Intent::OpenFile => self.open_file_dialog(),
            Intent::SaveFile => self.save_file(),
            Intent::SaveAs => self.save_as_dialog(),
            Intent::CloseFile => {
                self.content.clear();
                self.file_path.clear();
                self.status = "Файл закрыт.".to_string();
            }
            Intent::Exit => std::process::exit(0),
            Intent::Format => {
                if self.content.is_empty() {
                    self.status = "Редактор пуст. Нечего форматировать.".to_string();
                } else {
                    self.status = "Форматирование...".to_string();
                    match format_code_with_settings(&self.content, &self.format_settings) {
                        Ok(formatted) => {
                            self.content = formatted;
                            self.status = "Форматирование завершено".to_string();
                        }
                        Err(e) => self.status = format!("Ошибка: {}", e),
                    }
                }
            }
            Intent::Validate => {
                if self.content.is_empty() {
                    self.status = "Редактор пуст. Нечего проверять.".to_string();
                } else {
                    match validate_code(&self.content) {
                        Ok(msgs) => {
                            if msgs.is_empty() {
                                self.status = "Ошибок не найдено".to_string();
                            } else {
                                let has_err = msgs.iter().any(|e| e.severity == Severity::Error);
                                let level = if has_err {
                                    "ошибок"
                                } else {
                                    "предупреждений"
                                };
                                self.status = format!("Найдено {} {}", msgs.len(), level);
                            }
                        }
                        Err(e) => self.status = format!("Ошибка: {}", e),
                    }
                }
            }
            Intent::ToggleSettings => {
                self.settings_open = !self.settings_open;
            }
            Intent::SetRenumberStep(step) => {
                self.format_settings.renumber_step = *step;
                self.save_settings();
            }
            Intent::SetSkipEmptyLines(skip) => {
                self.format_settings.skip_empty_lines = *skip;
                self.save_settings();
            }
        }
    }

    fn open_file_dialog(&mut self) {
        if self.is_busy {
            return;
        }
        self.is_busy = true;
        let file = rfd::FileDialog::new()
            .add_filter("G-Code", &["txt", "nc", "cnc", "gcode", "ngc"])
            .pick_file();
        self.is_busy = false;
        if let Some(path) = file {
            let path_str = path.to_string_lossy().to_string();
            let content = std::fs::read_to_string(&path).ok().map(|s| {
                s.replace("\r\n", "\n")
                    .replace("\r", "\n")
                    .trim_start_matches('\u{feff}')
                    .to_string()
            });

            match content {
                Some(text) => {
                    let lines = text.lines().count();
                    self.content = text;
                    self.file_path = path_str;
                    self.status = format!(
                        "Загружен: {} ({} строк)",
                        std::path::Path::new(&self.file_path)
                            .file_name()
                            .map(|n| n.to_string_lossy())
                            .unwrap_or_default(),
                        lines,
                    );
                }
                None => {
                    self.status = "Ошибка чтения файла".to_string();
                }
            }
        }
    }

    fn save_file(&mut self) {
        if self.file_path.is_empty() {
            self.save_as_dialog();
            return;
        }
        match std::fs::write(&self.file_path, &self.content) {
            Ok(_) => self.status = "Сохранено".to_string(),
            Err(e) => self.status = format!("Ошибка: {}", e),
        }
    }

    fn save_as_dialog(&mut self) {
        if self.is_busy {
            return;
        }
        self.is_busy = true;
        let file = rfd::FileDialog::new()
            .add_filter("G-Code", &["nc", "cnc", "txt", "gcode"])
            .save_file();
        self.is_busy = false;
        if let Some(path) = file {
            let path_str = path.to_string_lossy().to_string();
            match std::fs::write(&path, &self.content) {
                Ok(_) => {
                    self.file_path = path_str;
                    self.status = "Сохранено".to_string();
                }
                Err(e) => self.status = format!("Ошибка: {}", e),
            }
        }
    }
}

// --- Сохранение и загрузка настроек ---

/// Путь к файлу настроек
fn settings_path() -> std::path::PathBuf {
    let mut path = if let Some(home) = std::env::var("HOME").ok() {
        std::path::PathBuf::from(home)
    } else {
        std::path::PathBuf::from(".")
    };
    path.push(".config");
    path.push("gcode-editor");
    path.push("settings.json");
    path
}

impl Model {
    /// Сохраняет настройки в файл
    pub fn save_settings(&self) {
        let path = settings_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.format_settings) {
            let _ = std::fs::write(&path, json);
        }
    }

    /// Загружает настройки из файла
    pub fn load_settings(&mut self) {
        let path = settings_path();
        if let Ok(json) = std::fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str::<super::model::FormatSettings>(&json) {
                self.format_settings = settings;
            }
        }
    }
}

// --- Вспомогательные функции пайплайна ---

fn format_code_with_settings(
    input: &str,
    settings: &super::model::FormatSettings,
) -> anyhow::Result<String> {
    use anyhow::Context;
    let tokens = crate::infrastructure::lexer::tokenize(input);
    let mut parser = crate::application::Parser::new(tokens);
    let program = parser.parse_program().context("Ошибка парсинга G-кода")?;
    let errors = crate::application::validate(&program);
    if errors.iter().any(|e| e.severity == Severity::Error) {
        anyhow::bail!("Найдено {} ошибок. Форматирование отменено.", errors.len());
    }

    let config = crate::application::FormatConfig {
        uppercase_codes: true,
        decimal_places: 5,
        renumber_step: settings.renumber_step,
        skip_empty_lines: settings.skip_empty_lines,
        ..Default::default()
    };

    let fmt = crate::application::Formatter::new(config);
    Ok(fmt.format_program(&program))
}

fn validate_code(input: &str) -> anyhow::Result<Vec<crate::shared::ValidationMessage>> {
    use anyhow::Context;
    let tokens = crate::infrastructure::lexer::tokenize(input);
    let mut parser = crate::application::Parser::new(tokens);
    let program = parser.parse_program().context("Ошибка парсинга G-кода")?;
    Ok(crate::application::validate(&program))
}
