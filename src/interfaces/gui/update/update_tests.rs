use crate::interfaces::gui::intent::Intent;
use crate::interfaces::gui::model::{Model, PendingAction};

/// Блокировка для глобального состояния i18n::LANG.
/// Все тесты, меняющие язык, должны использовать эту блокировку.
fn with_i18n_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap()
}

fn make_model() -> Model {
    let mut m = Model::default();
    m.set_content("G0 X10 Y20".to_string());
    m.set_file_path("/path/to/file.nc".to_string());
    m.set_modified(true);
    m
}

// -----------------------------------------------------------------------
// CloseFile
// -----------------------------------------------------------------------

#[test]
fn test_close_file_modified_shows_dialog() {
    let mut m = make_model();
    m.apply(&Intent::CloseFile);
    assert!(m.show_exit_dialog());
    assert_eq!(m.pending_action(), Some(&PendingAction::CloseFile));
    assert_eq!(m.content(), "G0 X10 Y20"); // не очищается до подтверждения
}

#[test]
fn test_close_file_not_modified_clears() {
    let mut m = Model::default();
    m.set_content("G0 X10".to_string());
    m.set_file_path("/path/file.nc".to_string());
    m.set_modified(false);
    m.apply(&Intent::CloseFile);
    assert!(!m.show_exit_dialog());
    assert!(m.content().is_empty());
    assert!(m.file_path().is_empty());
    assert!(!m.modified());
}

#[test]
fn test_close_file_no_path_clears() {
    let mut m = Model::default();
    m.set_content("G0 X10".to_string());
    m.set_modified(true);
    // Если file_path пуст — сразу очищаем, без диалога
    assert!(m.file_path().is_empty());
    m.apply(&Intent::CloseFile);
    assert!(!m.show_exit_dialog());
    assert!(m.content().is_empty());
}

// -----------------------------------------------------------------------
// Exit
// -----------------------------------------------------------------------

#[test]
fn test_exit_modified_shows_dialog() {
    let mut m = make_model();
    m.apply(&Intent::Exit);
    assert!(m.show_exit_dialog());
    assert_eq!(m.pending_action(), Some(&PendingAction::Exit));
}

// Exit без modified вызывает std::process::exit — не тестируем

// -----------------------------------------------------------------------
// ToggleSettings
// -----------------------------------------------------------------------

#[test]
fn test_toggle_settings_twice() {
    let mut m = Model::default();
    assert!(!m.settings_open());
    m.apply(&Intent::ToggleSettings);
    assert!(m.settings_open());
    m.apply(&Intent::ToggleSettings);
    assert!(!m.settings_open());
}

// -----------------------------------------------------------------------
// SetRenumberStep
// -----------------------------------------------------------------------

#[test]
fn test_set_renumber_step() {
    let mut m = Model::default();
    assert_eq!(m.format_settings().renumber_step, 1);
    m.apply(&Intent::SetRenumberStep(10));
    assert_eq!(m.format_settings().renumber_step, 10);
    m.apply(&Intent::SetRenumberStep(100));
    assert_eq!(m.format_settings().renumber_step, 100);
}

// -----------------------------------------------------------------------
// SetSkipEmptyLines
// -----------------------------------------------------------------------

#[test]
fn test_set_skip_empty_lines() {
    let mut m = Model::default();
    assert!(m.format_settings().skip_empty_lines);
    m.apply(&Intent::SetSkipEmptyLines(false));
    assert!(!m.format_settings().skip_empty_lines);
    m.apply(&Intent::SetSkipEmptyLines(true));
    assert!(m.format_settings().skip_empty_lines);
}

// -----------------------------------------------------------------------
// CancelAction
// -----------------------------------------------------------------------

#[test]
fn test_cancel_action() {
    let mut m = make_model();
    m.apply(&Intent::CloseFile); // открыли диалог
    assert!(m.show_exit_dialog());
    assert_eq!(m.pending_action(), Some(&PendingAction::CloseFile));

    m.apply(&Intent::CancelAction);
    assert!(!m.show_exit_dialog());
    assert_eq!(m.pending_action(), None);
}

// -----------------------------------------------------------------------
// SetLanguage
// -----------------------------------------------------------------------

#[test]
fn test_set_language_en() {
    let _lock = with_i18n_lock();
    let mut m = Model::default();
    assert_eq!(m.format_settings().language, "ru");
    m.apply(&Intent::SetLanguage("en".to_string()));
    assert_eq!(m.format_settings().language, "en");
    assert_eq!(crate::shared::i18n::current_lang(), "en");
    // Сброс
    crate::shared::i18n::set_lang("ru");
}

#[test]
fn test_set_language_ru() {
    let _lock = with_i18n_lock();
    let mut m = Model::default();
    m.apply(&Intent::SetLanguage("en".to_string()));
    m.apply(&Intent::SetLanguage("ru".to_string()));
    assert_eq!(m.format_settings().language, "ru");
    assert_eq!(crate::shared::i18n::current_lang(), "ru");
}

// -----------------------------------------------------------------------
// Intent'ы которые ничего не делают в update.rs
// -----------------------------------------------------------------------

#[test]
fn test_format_validate_noop() {
    // Format, Validate, OpenFile, SaveFile, SaveAs, ConfirmSave,
    // DiscardAndContinue — не меняют модель в apply(),
    // их обработка целиком в app.rs
    let mut m = make_model();
    let original = m.status().to_string();
    m.apply(&Intent::Format);
    m.apply(&Intent::Validate);
    m.apply(&Intent::OpenFile);
    m.apply(&Intent::SaveFile);
    m.apply(&Intent::SaveAs);
    m.apply(&Intent::ConfirmSave);
    m.apply(&Intent::DiscardAndContinue);
    assert_eq!(m.status(), original);
}
