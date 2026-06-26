//! AppState — состояние приложения, хранимое в StateStore.
//!
//! Аналог gui::model::Model, но:
//! - Clone + Send + Sync + 'static (требование StateStore)
//! - без мутирующих сеттеров (мутация только через store.update())
//! - PartialEq для оптимизации repaint

use crate::interfaces::gui::intent::AxisSwapMode;
use crate::interfaces::gui::model::FormatSettings;

/// Состояние приложения G-Code Editor под Android.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AppState {
    // ── Основные данные ──
    pub content: String,
    pub file_path: String,
    pub status: String,
    pub is_busy: bool,
    pub modified: bool,
    pub error_lines: Vec<usize>,

    // ── Настройки форматирования ──
    pub format_settings: FormatSettings,

    // ── Окна и диалоги ──
    pub settings_open: bool,
    pub shortcuts_open: bool,
    pub show_exit_dialog: bool,
    pub pending_action: Option<PendingActionUi>,
    pub save_and_exec: Option<PendingActionUi>,

    // ── Поиск ──
    pub search_open: bool,
    pub search_query: String,
    pub search_index: usize,
    pub search_matches: Vec<usize>,
    pub search_last_query: String,
    pub search_focus_needed: bool,

    // ── Замена ──
    pub replace_open: bool,
    pub replace_find: String,
    pub replace_with: String,
    pub replace_index: usize,
    pub replace_matches: Vec<usize>,
    pub replace_last_find: String,
    pub replace_focus_needed: bool,

    // ── Замена осей ──
    pub axis_swap_open: bool,
    pub axis_swap_mode: AxisSwapMode,
    pub axis_swap_axis1: String,
    pub axis_swap_axis2: String,
    pub axis_invert_axis: String,

    // ── Android ──
    pub drawer_open: bool,
    pub editor_needs_focus: bool,
}

/// Действие, ожидающее подтверждения пользователя (копия PendingAction из model)
#[derive(Debug, Clone, PartialEq)]
pub enum PendingActionUi {
    Exit,
    CloseFile,
    OpenNewFile,
}

impl From<AppState> for FormatSettings {
    fn from(state: AppState) -> Self {
        state.format_settings
    }
}
