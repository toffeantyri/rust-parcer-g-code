//! Model — состояние редактора G-кода

use serde::{Deserialize, Serialize};

/// Состояние приложения
#[derive(Default)]
pub struct Model {
    content: String,
    file_path: String,
    status: String,
    is_busy: bool,
    settings_open: bool,
    format_settings: FormatSettings,
    modified: bool,
    show_exit_dialog: bool,
    shortcuts_open: bool,
    pending_action: Option<PendingAction>,
    save_and_exec: Option<PendingAction>,
    error_lines: Vec<usize>,
    editor_needs_focus: bool,
}

impl Model {
    pub fn content(&self) -> &str {
        &self.content
    }
    pub fn file_path(&self) -> &str {
        &self.file_path
    }
    pub fn status(&self) -> &str {
        &self.status
    }
    pub fn is_busy(&self) -> bool {
        self.is_busy
    }
    pub fn settings_open(&self) -> bool {
        self.settings_open
    }
    pub fn format_settings(&self) -> &FormatSettings {
        &self.format_settings
    }
    pub fn modified(&self) -> bool {
        self.modified
    }
    pub fn show_exit_dialog(&self) -> bool {
        self.show_exit_dialog
    }
    pub fn shortcuts_open(&self) -> bool {
        self.shortcuts_open
    }
    pub fn pending_action(&self) -> Option<&PendingAction> {
        self.pending_action.as_ref()
    }
    pub fn save_and_exec(&self) -> Option<&PendingAction> {
        self.save_and_exec.as_ref()
    }
    pub fn error_lines(&self) -> &[usize] {
        &self.error_lines
    }
    pub fn editor_needs_focus(&self) -> bool {
        self.editor_needs_focus
    }
}

impl Model {
    /// Мутирует модель. Единственный способ изменить состояние.
    /// Используется для прямых мутаций из app.rs (handle_event, handle_intent).
    /// Не-pub поля, чтобы гарантировать MVI.
    pub fn set_content(&mut self, v: String) {
        self.content = v;
    }
    pub fn set_file_path(&mut self, v: String) {
        self.file_path = v;
    }
    pub fn set_status(&mut self, v: String) {
        self.status = v;
    }
    pub fn set_is_busy(&mut self, v: bool) {
        self.is_busy = v;
    }
    pub fn set_settings_open(&mut self, v: bool) {
        self.settings_open = v;
    }
    pub fn set_format_settings(&mut self, v: FormatSettings) {
        self.format_settings = v;
    }
    pub fn set_modified(&mut self, v: bool) {
        self.modified = v;
    }
    pub fn set_show_exit_dialog(&mut self, v: bool) {
        self.show_exit_dialog = v;
    }
    pub fn set_shortcuts_open(&mut self, v: bool) {
        self.shortcuts_open = v;
    }
    pub fn set_pending_action(&mut self, v: Option<PendingAction>) {
        self.pending_action = v;
    }
    pub fn set_save_and_exec(&mut self, v: Option<PendingAction>) {
        self.save_and_exec = v;
    }
    pub fn set_error_lines(&mut self, v: Vec<usize>) {
        self.error_lines = v;
    }
    pub fn set_editor_needs_focus(&mut self, v: bool) {
        self.editor_needs_focus = v;
    }
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
