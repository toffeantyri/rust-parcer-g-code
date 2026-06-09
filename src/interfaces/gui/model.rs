//! Model — состояние редактора G-кода

use serde::{Deserialize, Serialize};

/// Состояние приложения
#[derive(Default)]
pub struct Model {
    /// Содержимое редактора G-кода
    pub content: String,
    /// Путь к текущему файлу
    pub file_path: String,
    /// Текст в строке состояния
    pub status: String,
    /// Флаг: диалог открытия/сохранения уже показан
    pub is_busy: bool,
    /// Открыто ли окно настроек форматирования
    pub settings_open: bool,
    /// Настройки форматирования
    pub format_settings: FormatSettings,
}

/// Настройки форматирования G-кода
#[derive(Clone, Serialize, Deserialize)]
pub struct FormatSettings {
    /// Шаг перенумерации кадров (1, 10, 100...)
    pub renumber_step: u32,
    /// Пропускать пустые строки при перенумерации
    pub skip_empty_lines: bool,
}

impl Default for FormatSettings {
    fn default() -> Self {
        Self {
            renumber_step: 1,
            skip_empty_lines: true,
        }
    }
}
