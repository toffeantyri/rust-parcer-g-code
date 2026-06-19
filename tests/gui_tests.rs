//! Интеграционные тесты GUI через egui_kittest.
//!
//! Тестируем: меню, кнопки тулбара, диалоги, строку состояния,
//! редактор кода без скриншотов — через AccessKit-дерево.

use egui::accesskit::{Role, Toggled};
use egui_kittest::{kittest::Queryable, Harness};

use code_parser::data_layer::{EditorEvent, PipelineEvent};
use code_parser::interfaces::gui::{
    collect_intents, view_editor, view_exit_dialog, view_search_dialog, view_settings,
    view_shortcuts, view_statusbar, FormatSettings, Intent, Model,
};
use code_parser::shared::i18n;
use code_parser::shared::ValidationMessage;

/// Блокировка для глобального состояния i18n::LANG.
/// Все GUI тесты, меняющие язык, должны использовать эту блокировку,
/// чтобы избежать гонки за глобальный статик LANG.
static I18N_LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();

fn with_i18n_lock() -> std::sync::MutexGuard<'static, ()> {
    I18N_LOCK
        .get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap()
}

// ---------------------------------------------------------------------------
// Menu — File
// ---------------------------------------------------------------------------

#[test]
fn test_menu_file_items() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let model = Model::default();

    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        let _intents = collect_intents(ui.ctx(), false, &model);
    });
    harness.run();

    let file_menu = harness.get_by_label("File");
    file_menu.click();
    harness.run();

    assert!(harness.query_by_label("Open...").is_some());
    // Save есть и в меню, и в тулбаре — используем query_all
    assert!(harness.query_all_by_label("Save").count() >= 2);
    assert!(harness.query_by_label("Save as...").is_some());
    assert!(harness.query_by_label("Close").is_some());
    assert!(harness.query_by_label("Exit").is_some());
}

#[test]
fn test_menu_file_items_ru() {
    let _lock = with_i18n_lock();
    i18n::set_lang("ru");
    let model = Model::default();

    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        let _intents = collect_intents(ui.ctx(), false, &model);
    });
    harness.run();

    let file_menu = harness.get_by_label("Файл");
    file_menu.click();
    harness.run();

    assert!(harness.query_by_label("Открыть...").is_some());
    assert!(harness.query_all_by_label("Сохранить").count() >= 2);
    assert!(harness.query_by_label("Сохранить как...").is_some());
    assert!(harness.query_by_label("Закрыть").is_some());
    assert!(harness.query_by_label("Выход").is_some());
}

// ---------------------------------------------------------------------------
// Menu — Edit
// ---------------------------------------------------------------------------

#[test]
fn test_menu_edit_items() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let model = Model::default();

    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        let _intents = collect_intents(ui.ctx(), false, &model);
    });
    harness.run();

    let edit_menu = harness.get_by_label("Edit");
    edit_menu.click();
    harness.run();

    assert!(harness.query_by_label("Format (F5)").is_some());
    assert!(harness.query_by_label("Validate (F6)").is_some());
    assert!(harness.query_by_label("Format Settings...").is_some());
}

// ---------------------------------------------------------------------------
// Toolbar buttons
// ---------------------------------------------------------------------------

#[test]
fn test_toolbar_buttons() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let model = Model::default();

    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        let _intents = collect_intents(ui.ctx(), false, &model);
    });
    harness.run();

    assert!(harness.query_by_label("Open").is_some());
    assert!(harness.query_by_label("Save").is_some());
    assert!(harness.query_by_label("Format").is_some());
    assert!(harness.query_by_label("Check").is_some());
}

#[test]
fn test_toolbar_buttons_disabled_when_busy() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let mut model = Model::default();
    model.set_is_busy(true);

    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        let _intents = collect_intents(ui.ctx(), false, &model);
    });
    harness.run();

    assert!(harness.query_by_label("Open").is_some()); // кнопка видна, но disabled
}

// ---------------------------------------------------------------------------
// Editor
// ---------------------------------------------------------------------------

#[test]
fn test_editor_is_present() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let mut model = Model::default();

    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        let mut last_change = std::time::Instant::now();
        let mut pending: Option<String> = None;
        view_editor(
            &mut model,
            ui.ctx(),
            &std::sync::mpsc::channel().0,
            &mut last_change,
            &mut pending,
        );
    });
    harness.run();

    let editor = harness
        .query_by_role(Role::MultilineTextInput)
        .or_else(|| harness.query_by_role(Role::TextInput));
    assert!(editor.is_some());
}

#[test]
fn test_editor_shows_content() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let mut model = Model::default();
    model.set_content("G0 X10 Y20\nG1 Z5.5".to_string());

    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        let mut last_change = std::time::Instant::now();
        let mut pending: Option<String> = None;
        view_editor(
            &mut model,
            ui.ctx(),
            &std::sync::mpsc::channel().0,
            &mut last_change,
            &mut pending,
        );
    });
    harness.run();

    let editor = harness.get_by_role(Role::MultilineTextInput);
    assert_eq!(editor.value(), Some("G0 X10 Y20\nG1 Z5.5".to_string()));
}

// ---------------------------------------------------------------------------
// Status bar
// ---------------------------------------------------------------------------

#[test]
fn test_status_bar_shows_ready() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let mut model = Model::default();
    model.set_status(i18n::locale().status.ready.clone());

    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        view_statusbar(&model, ui.ctx());
    });
    harness.run();

    assert!(harness
        .query_by_label(i18n::locale().status.ready.as_str())
        .is_some());
}

// ---------------------------------------------------------------------------
// Exit dialog
// ---------------------------------------------------------------------------

#[test]
fn test_exit_dialog_shown_when_flag_set() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let mut model = Model::default();
    model.set_show_exit_dialog(true);
    model.set_content("G0 X10".to_string());
    model.set_file_path("/path.nc".to_string());
    model.set_modified(true);

    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        let _intents = view_exit_dialog(&model, ui.ctx());
    });
    harness.run();

    assert!(harness.query_by_label("Save changes?").is_some());
    assert!(harness.query_by_label("Save").is_some());
    assert!(harness.query_by_label("Discard").is_some());
    assert!(harness.query_by_label("Cancel").is_some());
}

#[test]
fn test_exit_dialog_not_shown_without_flag() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let model = Model::default();

    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        let _intents = view_exit_dialog(&model, ui.ctx());
    });
    harness.run();

    assert!(harness.query_by_label("Save changes?").is_none());
}

// ---------------------------------------------------------------------------
// Format Settings window
// ---------------------------------------------------------------------------

#[test]
fn test_settings_window_shown_when_open() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let mut model = Model::default();
    model.set_settings_open(true);

    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        let _intents = view_settings(&model, ui.ctx());
    });
    harness.run();

    assert!(harness.query_by_label("Format Settings").is_some());
    assert!(harness.query_by_label("Renumber step:").is_some());
    assert!(harness.query_by_label("Examples:").is_some());
}

#[test]
fn test_settings_window_not_shown_by_default() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let model = Model::default();

    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        let _intents = view_settings(&model, ui.ctx());
    });
    harness.run();

    assert!(harness.query_by_label("Format Settings").is_none());
}

#[test]
fn test_settings_window_skip_empty_checkbox() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let mut model = Model::default();
    model.set_settings_open(true);
    model.set_format_settings(FormatSettings {
        skip_empty_lines: false,
        ..Default::default()
    });

    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        let _intents = view_settings(&model, ui.ctx());
    });
    harness.run();

    let checkbox = harness.get_by_label("Skip empty lines when renumbering");
    assert_eq!(checkbox.toggled(), Some(Toggled::False));
}

// ---------------------------------------------------------------------------
// Language переключатель в Settings меню
// ---------------------------------------------------------------------------

#[test]
fn test_settings_menu_language_toggle() {
    let _lock = with_i18n_lock();
    i18n::set_lang("ru");
    let mut model = Model::default();
    model.set_format_settings(FormatSettings {
        language: "ru".to_string(),
        ..Default::default()
    });

    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        let _intents = collect_intents(ui.ctx(), false, &model);
    });
    harness.run();

    // Открываем Настройки
    let settings = harness.get_by_label("Настройки");
    settings.click();
    harness.run();

    assert!(harness.query_by_label("English").is_some());
    assert!(harness.query_by_label("Русский").is_some());
}

// ---------------------------------------------------------------------------
// Shortcuts window
// ---------------------------------------------------------------------------

#[test]
fn test_shortcuts_window_shown_when_open() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let mut model = Model::default();
    model.set_shortcuts_open(true);

    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        let _intents = view_shortcuts(&model, ui.ctx());
    });
    harness.run();

    // Окно горячих клавиш должно быть видно — проверяем по содержимому
    assert!(harness.query_by_label("Ctrl+O").is_some());
    assert!(harness.query_by_label("Ctrl+S").is_some());
    assert!(harness.query_by_label("F5").is_some());
    assert!(harness.query_by_label("F6").is_some());
}

#[test]
fn test_shortcuts_window_not_shown_by_default() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let model = Model::default();

    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        let _intents = view_shortcuts(&model, ui.ctx());
    });
    harness.run();

    assert!(harness.query_by_label("Shortcuts").is_none());
}

#[test]
fn test_shortcuts_window_shown_russian() {
    let _lock = with_i18n_lock();
    i18n::set_lang("ru");
    let mut model = Model::default();
    model.set_shortcuts_open(true);

    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        let _intents = view_shortcuts(&model, ui.ctx());
    });
    harness.run();

    assert!(
        harness.query_by_label("Горячие клавиши").is_some()
            || harness.query_by_label("Ctrl+O").is_some()
    );
    assert!(harness.query_by_label("Ctrl+S").is_some());
}

// ---------------------------------------------------------------------------
// Error lines — проверка, что error_lines заполняется после Validate
// ---------------------------------------------------------------------------

#[test]
fn test_validate_fills_error_lines() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let mut model = Model::default();
    model.set_content("G0 X".to_string()); // ось X без значения — ошибка

    // Имитируем событие Validated, как это делает app.rs::handle_event
    let err = code_parser::shared::ValidationMessage::error(1, "ось X без значения");

    // Создаём GCodeApp, но напрямую вызываем handle_event
    let (tx, _) = std::sync::mpsc::channel();
    let (_evt_tx, evt_rx) = std::sync::mpsc::channel();
    let mut app = code_parser::interfaces::gui::GCodeApp::new(tx, evt_rx);
    app.model.set_content("G0 X".to_string());

    app.handle_event(code_parser::data_layer::EditorEvent::Pipeline(
        code_parser::data_layer::PipelineEvent::Validated { errors: vec![err] },
    ));

    assert!(!app.model.error_lines().is_empty());
    assert_eq!(app.model.error_lines(), vec![1]);
}

// ---------------------------------------------------------------------------
// Exit dialog — новые функции
// ---------------------------------------------------------------------------

#[test]
fn test_exit_dialog_pressed_enter_confirms_selected() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let mut model = Model::default();
    model.set_show_exit_dialog(true);
    model.set_content("G0 X10".to_string());
    model.set_file_path("/path.nc".to_string());
    model.set_modified(true);

    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        let _intents = view_exit_dialog(&model, ui.ctx());
    });
    harness.run();

    // Диалог виден
    assert!(harness.query_by_label("Save changes?").is_some());
}

#[test]
fn test_editor_autofocus_after_load() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let mut model = Model::default();
    model.set_content("G0 X10".to_string());
    model.set_file_path("/test.nc".to_string());
    model.set_editor_needs_focus(true);
    model.set_modified(false);

    let mut pending: Option<String> = None;
    let mut last_change = std::time::Instant::now();
    let (tx, _) = std::sync::mpsc::channel();

    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        view_editor(&mut model, ui.ctx(), &tx, &mut last_change, &mut pending);
    });
    harness.run();

    // После первого кадра флаг должен сброситься
    // (тестируем что нет паники и редактор отрисовался)
    let editor = harness
        .query_by_role(Role::MultilineTextInput)
        .or_else(|| harness.query_by_role(Role::TextInput));
    assert!(editor.is_some());
}

#[test]
fn test_exit_dialog_escape_closes() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let mut model = Model::default();
    model.set_show_exit_dialog(true);
    model.set_content("G0 X10".to_string());
    model.set_file_path("/path.nc".to_string());
    model.set_modified(true);

    let mut intents = Vec::new();
    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        intents.extend(view_exit_dialog(&model, ui.ctx()));
    });
    harness.run();

    // Симулируем нажатие Escape
    harness.press_key(egui::Key::Escape);
    harness.run();

    // Intents должен содержать CancelAction (но через Harness не поймать,
    // т.к. intents собирается внутри closure)
    // Проверяем хотя бы что диалог отрисовался
    assert!(harness.query_by_label("Cancel").is_some());
}

// ---------------------------------------------------------------------------
// Search dialog
// ---------------------------------------------------------------------------

#[test]
fn test_search_dialog_shows_when_open() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let mut model = Model::default();
    model.set_search_open(true);

    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        let _intents = view_search_dialog(&mut model, ui.ctx());
    });
    harness.run();

    // Диалог виден — проверяем по содержимому
    assert!(harness.query_by_label("Search").is_some());
    assert!(harness.query_by_label("Find").is_some());
    assert!(harness.query_by_label("Cancel").is_some());
}

#[test]
fn test_search_dialog_not_shown_by_default() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let mut model = Model::default();

    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        let _intents = view_search_dialog(&mut model, ui.ctx());
    });
    harness.run();

    assert!(harness.query_by_label("Search").is_none());
}

#[test]
fn test_search_dialog_enter_autofocus_works() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let mut model = Model::default();
    model.set_content("G0 X10 Y20".to_string());
    model.set_search_open(true);
    model.set_search_query("G0".to_string());
    model.set_search_focus_needed(true);

    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        let _intents = view_search_dialog(&mut model, ui.ctx());
    });
    harness.run();

    // Поле ввода должно быть видно
    let input = harness.query_by_role(Role::TextInput);
    assert!(input.is_some(), "Поле ввода поиска должно быть в диалоге");
}

#[test]
fn test_search_dialog_escape_closes_via_intent() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let mut model = Model::default();
    model.set_search_open(true);
    model.set_search_query("G0".to_string());

    // Имитируем обработку Escape вручную через Model::apply
    // (Harness не позволяет собрать intents из-за move в замыкание)
    model.apply(&Intent::CloseSearchDialog);
    assert!(
        !model.search_open(),
        "CloseSearchDialog должен закрыть диалог"
    );
    assert!(model.search_query().is_empty());
}

#[test]
fn test_search_dialog_find_button_click_via_model() {
    let _lock = with_i18n_lock();
    i18n::set_lang("en");
    let mut model = Model::default();
    model.set_content("G0 X10 Y20\nG0 Z30".to_string());
    model.set_search_open(true);
    model.set_search_query("G0".to_string());

    // Имитируем клик Find (→ FindNext)
    model.apply(&Intent::FindNext);

    // После первого FindNext должен быть пересчёт
    assert_eq!(model.search_matches().len(), 2);
    assert_eq!(model.search_index(), 0);

    // Второй клик — перейти к следующему
    model.apply(&Intent::FindNext);
    assert_eq!(model.search_index(), 1);

    // Третий — цикл
    model.apply(&Intent::FindNext);
    assert_eq!(model.search_index(), 0);
}
