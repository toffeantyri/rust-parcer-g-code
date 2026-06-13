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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_lang_is_ru() {
        // Язык по умолчанию — русский
        assert_eq!(current_lang(), "ru");
        assert_eq!(&locale().menu.file, "Файл");
        assert_eq!(
            &locale().status.ready,
            "Готов к работе. Откройте файл G-кода."
        );
    }

    #[test]
    fn test_set_lang_en() {
        set_lang("en");
        assert_eq!(current_lang(), "en");
        assert_eq!(&locale().menu.file, "File");
        assert_eq!(&locale().status.ready, "Ready. Open a G-code file.");
        // Сброс обратно на ru, чтобы не влиять на другие тесты
        set_lang("ru");
        assert_eq!(current_lang(), "ru");
    }

    #[test]
    fn test_set_lang_back_and_forth() {
        set_lang("en");
        assert_eq!(current_lang(), "en");
        set_lang("ru");
        assert_eq!(current_lang(), "ru");
        set_lang("en-US");
        assert_eq!(current_lang(), "en");
        set_lang("fr"); // неизвестный язык — падает на ru
        assert_eq!(current_lang(), "ru");
    }

    #[test]
    fn test_locale_switch_affects_all_categories() {
        set_lang("en");
        assert_eq!(&locale().menu.settings, "Settings");
        assert_eq!(&locale().toolbar.open, "Open");
        assert_eq!(&locale().dialog.exit_title, "Save changes?");
        assert_eq!(&locale().settings.title, "Format Settings");
        set_lang("ru");
    }

    #[test]
    fn test_fmt_save_error_ru() {
        set_lang("ru");
        let msg = fmt_save_error("Permission denied");
        assert_eq!(msg, "Ошибка сохранения: Permission denied");
    }

    #[test]
    fn test_fmt_save_error_en() {
        set_lang("en");
        let msg = fmt_save_error("Permission denied");
        assert_eq!(msg, "Save error: Permission denied");
        set_lang("ru");
    }

    #[test]
    fn test_fmt_error_ru() {
        set_lang("ru");
        let msg = fmt_error("Syntax error");
        assert_eq!(msg, "Ошибка: Syntax error");
    }

    #[test]
    fn test_fmt_error_en() {
        set_lang("en");
        let msg = fmt_error("Syntax error");
        assert_eq!(msg, "Error: Syntax error");
        set_lang("ru");
    }

    #[test]
    fn test_fmt_errors_found_ru() {
        set_lang("ru");
        let msg = fmt_errors_found(3, "ось X без значения", 5);
        assert_eq!(
            msg,
            "Найдено 3 ошибок. Первая: ось X без значения [строка 5]"
        );
    }

    #[test]
    fn test_fmt_errors_found_en() {
        set_lang("en");
        let msg = fmt_errors_found(1, "axis X without value", 2);
        assert_eq!(msg, "Found 1 errors. First: axis X without value [line 2]");
        set_lang("ru");
    }

    #[test]
    fn test_fmt_saved_ru() {
        set_lang("ru");
        let msg = fmt_saved("/path/file.nc");
        assert_eq!(msg, "Сохранено: /path/file.nc");
    }

    #[test]
    fn test_fmt_saved_en() {
        set_lang("en");
        let msg = fmt_saved("/path/file.nc");
        assert_eq!(msg, "Saved: /path/file.nc");
        set_lang("ru");
    }
}
