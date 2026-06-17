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
    /// Был ли изменён текст после последнего сохранения/открытия
    pub modified: bool,
    /// Флаг: показать диалог подтверждения выхода
    pub show_exit_dialog: bool,
    /// Флаг: показать окно горячих клавиш
    pub shortcuts_open: bool,
    /// Действие, которое нужно выполнить после подтверждения диалога
    pub pending_action: Option<PendingAction>,
    /// Действие, ожидающее завершения сохранения в data layer
    pub save_and_exec: Option<PendingAction>,
}

/// Действие, ожидающее подтверждения пользователя
#[derive(Debug, Clone, PartialEq)]
pub enum PendingAction {
    Exit,
    CloseFile,
    OpenNewFile,
}

/// Настройки форматирования G-кода
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FormatSettings {
    /// Шаг перенумерации кадров (1, 10, 100...)
    pub renumber_step: u32,
    /// Пропускать пустые строки при перенумерации
    pub skip_empty_lines: bool,
    /// Язык интерфейса: "ru" или "en"
    pub language: String,
}

impl Default for FormatSettings {
    fn default() -> Self {
        Self {
            renumber_step: 1,
            skip_empty_lines: true,
            language: "ru".to_string(),
        }
    }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
