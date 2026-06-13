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
    /// Действие, которое нужно выполнить после подтверждения диалога
    pub pending_action: Option<PendingAction>,
    /// Флаг: после сохранения нужно выйти из программы
    pub exiting_after_save: bool,
    /// Флаг: после сохранения нужно закрыть файл (игнорировать Formatted с file_path)
    pub closing_after_save: bool,
}

/// Действие, ожидающее подтверждения пользователя
#[derive(Clone, PartialEq)]
pub enum PendingAction {
    Exit,
    CloseFile,
    OpenNewFile,
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
