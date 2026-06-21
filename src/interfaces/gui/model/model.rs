//! Model — состояние редактора G-кода

use serde::{Deserialize, Serialize};

use crate::interfaces::gui::intent::AxisSwapMode;

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
    /// Поиск: открыто ли окно поиска
    search_open: bool,
    /// Поиск: текущая строка запроса
    search_query: String,
    /// Поиск: индекс текущего вхождения (от 0)
    search_index: usize,
    /// Поиск: позиции всех вхождений (byte offset начала)
    search_matches: Vec<usize>,
    /// Поиск: предыдущий поисковый запрос (для сброса при изменении)
    search_last_query: String,
    /// Замена: открыто ли окно замены
    replace_open: bool,
    /// Замена: что ищем
    replace_find: String,
    /// Замена: на что заменяем
    replace_with: String,
    /// Замена: индекс текущего вхождения
    replace_index: usize,
    /// Замена: позиции всех вхождений
    replace_matches: Vec<usize>,
    /// Замена: предыдущий запрос поиска в окне замены
    replace_last_find: String,
    /// Флаг: запросить фокус поля ввода поиска при следующем кадре
    search_focus_needed: bool,
    /// Флаг: запросить фокус поля «Найти» в диалоге замены при следующем кадре
    replace_focus_needed: bool,
    /// Замена осей: открыто ли окно
    axis_swap_open: bool,
    /// Замена осей: режим (Swap или Invert)
    axis_swap_mode: AxisSwapMode,
    /// Замена осей: первая ось
    axis_swap_axis1: String,
    /// Замена осей: вторая ось
    axis_swap_axis2: String,
    /// Замена осей: ось для инвертирования (режим Invert)
    axis_invert_axis: String,
    /// Android Drawer: открыто ли боковое меню
    drawer_open: bool,
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
    pub fn search_open(&self) -> bool {
        self.search_open
    }
    pub fn search_query(&self) -> &str {
        &self.search_query
    }
    pub fn search_index(&self) -> usize {
        self.search_index
    }
    pub fn search_matches(&self) -> &[usize] {
        &self.search_matches
    }
    pub fn replace_open(&self) -> bool {
        self.replace_open
    }
    pub fn replace_find(&self) -> &str {
        &self.replace_find
    }
    pub fn replace_with(&self) -> &str {
        &self.replace_with
    }
    pub fn replace_index(&self) -> usize {
        self.replace_index
    }
    pub fn replace_matches(&self) -> &[usize] {
        &self.replace_matches
    }
    pub fn search_last_query(&self) -> &str {
        &self.search_last_query
    }
    pub fn replace_last_find(&self) -> &str {
        &self.replace_last_find
    }
    pub fn search_focus_needed(&self) -> bool {
        self.search_focus_needed
    }
    pub fn replace_focus_needed(&self) -> bool {
        self.replace_focus_needed
    }
    pub fn axis_swap_open(&self) -> bool {
        self.axis_swap_open
    }
    pub fn axis_swap_mode(&self) -> &AxisSwapMode {
        &self.axis_swap_mode
    }
    pub fn axis_swap_axis1(&self) -> &str {
        &self.axis_swap_axis1
    }
    pub fn axis_swap_axis2(&self) -> &str {
        &self.axis_swap_axis2
    }
    pub fn axis_invert_axis(&self) -> &str {
        &self.axis_invert_axis
    }
    pub fn flag_drawer_open(&self) -> bool {
        self.drawer_open
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
    pub fn set_search_open(&mut self, v: bool) {
        self.search_open = v;
    }
    pub fn set_search_query(&mut self, v: String) {
        self.search_query = v;
    }
    pub fn set_search_index(&mut self, v: usize) {
        self.search_index = v;
    }
    pub fn set_search_matches(&mut self, v: Vec<usize>) {
        self.search_matches = v;
    }
    pub fn set_search_last_query(&mut self, v: String) {
        self.search_last_query = v;
    }
    pub fn set_replace_open(&mut self, v: bool) {
        self.replace_open = v;
    }
    pub fn set_replace_find(&mut self, v: String) {
        self.replace_find = v;
    }
    pub fn set_replace_with(&mut self, v: String) {
        self.replace_with = v;
    }
    pub fn set_replace_index(&mut self, v: usize) {
        self.replace_index = v;
    }
    pub fn set_replace_matches(&mut self, v: Vec<usize>) {
        self.replace_matches = v;
    }
    pub fn set_replace_last_find(&mut self, v: String) {
        self.replace_last_find = v;
    }
    pub fn set_search_focus_needed(&mut self, v: bool) {
        self.search_focus_needed = v;
    }
    pub fn set_replace_focus_needed(&mut self, v: bool) {
        self.replace_focus_needed = v;
    }
    pub fn set_axis_swap_open(&mut self, v: bool) {
        self.axis_swap_open = v;
    }
    pub fn set_axis_swap_mode(&mut self, v: AxisSwapMode) {
        self.axis_swap_mode = v;
    }
    pub fn set_axis_swap_axis1(&mut self, v: String) {
        self.axis_swap_axis1 = v;
    }
    pub fn set_axis_swap_axis2(&mut self, v: String) {
        self.axis_swap_axis2 = v;
    }
    pub fn set_axis_invert_axis(&mut self, v: String) {
        self.axis_invert_axis = v;
    }
    pub fn set_drawer_open(&mut self, v: bool) {
        self.drawer_open = v;
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
