//! Модуль интернационализации: загрузка YAML-файлов локали
//! и переключение между русским и английским языками.

use serde::Deserialize;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::LazyLock;

static LANG: AtomicU8 = AtomicU8::new(0); // 0 = ru, 1 = en

// ---------------------------------------------------------------------------
// Структуры для десериализации YAML-файлов локали
// ---------------------------------------------------------------------------

#[derive(Deserialize, Clone)]
pub struct Locale {
    pub status: StatusStrings,
    pub menu: MenuStrings,
    pub toolbar: ToolbarStrings,
    pub dialog: DialogStrings,
    pub settings: SettingsStrings,
    pub search: SearchStrings,
    pub replace: ReplaceStrings,
}

#[derive(Deserialize, Clone)]
pub struct StatusStrings {
    pub ready: String,
    pub formatting: String,
    pub formatted: String,
    pub validating: String,
    pub no_errors: String,
    pub file_closed: String,
    pub saved: String,
    pub saved_as: String,
    pub file_opened: String,
    pub empty_editor: String,
    pub empty_validate: String,
    pub save_error: String,
    pub error_prefix: String,
    pub errors_found: String,
}

#[derive(Deserialize, Clone)]
pub struct MenuStrings {
    pub file: String,
    pub open: String,
    pub save: String,
    pub save_as: String,
    pub close: String,
    pub exit: String,
    pub edit: String,
    pub format: String,
    pub validate: String,
    pub format_settings: String,
    pub help: String,
    pub shortcuts: String,
    pub about: String,
    pub settings: String,
    pub shortcuts_title: String,
    pub language: String,
    pub lang_ru: String,
    pub lang_en: String,
    pub search: String,
    pub replace: String,
}

#[derive(Deserialize, Clone)]
pub struct ToolbarStrings {
    pub open: String,
    pub save: String,
    pub format: String,
    pub check: String,
}

#[derive(Deserialize, Clone)]
pub struct DialogStrings {
    pub exit_title: String,
    pub confirm_save: String,
    pub btn_save: String,
    pub btn_discard: String,
    pub btn_cancel: String,
}

#[derive(Deserialize, Clone)]
pub struct SettingsStrings {
    pub title: String,
    pub renumber_step: String,
    pub skip_empty: String,
    pub examples: String,
}

#[derive(Deserialize, Clone)]
pub struct SearchStrings {
    pub title: String,
    pub search_hint: String,
    pub btn_find: String,
    pub btn_cancel: String,
    pub not_found: String,
}

#[derive(Deserialize, Clone)]
pub struct ReplaceStrings {
    pub title: String,
    pub find_hint: String,
    pub replace_hint: String,
    pub btn_find: String,
    pub btn_replace: String,
    pub btn_replace_all: String,
    pub btn_cancel: String,
    pub not_found: String,
    pub replaced: String,
}

// ---------------------------------------------------------------------------
// Загрузка файлов локали (встраиваются на этапе компиляции)
// ---------------------------------------------------------------------------

static RU: LazyLock<Locale> = LazyLock::new(|| {
    serde_yaml::from_str(include_str!("locale/ru-RU.yml")).expect("Неверный ru-RU.yml")
});

static EN: LazyLock<Locale> = LazyLock::new(|| {
    serde_yaml::from_str(include_str!("locale/en-US.yml")).expect("Неверный en-US.yml")
});

// ---------------------------------------------------------------------------
// Публичный API
// ---------------------------------------------------------------------------

/// Устанавливает язык: `"ru"` или `"en"`
pub fn set_lang(lang: &str) {
    match lang {
        "en" | "en-US" => LANG.store(1, Ordering::Relaxed),
        _ => LANG.store(0, Ordering::Relaxed),
    }
}

/// Возвращает текущий язык: `"ru"` или `"en"`
pub fn current_lang() -> &'static str {
    match LANG.load(Ordering::Relaxed) {
        1 => "en",
        _ => "ru",
    }
}

/// Возвращает активную локаль
pub fn locale() -> &'static Locale {
    match LANG.load(Ordering::Relaxed) {
        1 => &EN,
        _ => &RU,
    }
}

// ---------------------------------------------------------------------------
// Вспомогательные функции для форматирования строк с placeholders
// ---------------------------------------------------------------------------

/// Форматирует ошибку сохранения
pub fn fmt_save_error(msg: &str) -> String {
    locale().status.save_error.replace("{msg}", msg)
}

/// Форматирует префикс ошибки
pub fn fmt_error(msg: &str) -> String {
    locale().status.error_prefix.replace("{msg}", msg)
}

/// Форматирует сообщение о количестве ошибок
pub fn fmt_errors_found(count: usize, msg: &str, line: usize) -> String {
    locale()
        .status
        .errors_found
        .replace("{count}", &count.to_string())
        .replace("{msg}", msg)
        .replace("{line}", &line.to_string())
}

/// Форматирует сообщение о сохранении
pub fn fmt_saved(path: &str) -> String {
    locale().status.saved_as.replace("{path}", path)
}

/// Блокировка для глобального LANG. Используется в тестах.
#[doc(hidden)]
pub fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap()
}

#[cfg(test)]
#[path = "i18n_tests.rs"]
mod tests;
