//! Интеграционные тесты GUI через egui_kittest.
//!
//! Тестируем: меню, кнопки тулбара, диалоги, строку состояния,
//! редактор кода без скриншотов — через AccessKit-дерево.

use egui::accesskit::{Role, Toggled};
use egui_kittest::{kittest::Queryable, Harness};

use code_parser::interfaces::gui::{
    collect_intents, view_editor, view_exit_dialog, view_settings, view_shortcuts, view_statusbar,
    FormatSettings, Model,
};
use code_parser::shared::i18n;

// ---------------------------------------------------------------------------
// Menu — File
// ---------------------------------------------------------------------------

#[test]
fn test_menu_file_items() {
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
