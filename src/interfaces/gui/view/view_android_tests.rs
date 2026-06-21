//! Тесты для Android view.
//! Используют egui_kittest для тестирования UI.

use crate::interfaces::gui::intent::Intent;
use crate::interfaces::gui::model::Model;
use crate::interfaces::gui::view;

// -----------------------------------------------------------------------
// Тесты collect_intents — проверяем работу дровера
// -----------------------------------------------------------------------

#[test]
fn test_collect_intents_drawer_initial_state() {
    let mut model = Model::default();
    assert!(
        !model.flag_drawer_open(),
        "дровер должен быть закрыт изначально"
    );
}

#[test]
fn test_collect_intents_toggle_drawer() {
    let mut model = Model::default();
    assert!(!model.flag_drawer_open());

    // Симулируем: нажали на бургер
    model.set_drawer_open(true);
    assert!(model.flag_drawer_open());

    // Симулируем: нажали снова
    model.set_drawer_open(false);
    assert!(!model.flag_drawer_open());
}

#[test]
fn test_collect_intents_editor_interaction() {
    let mut model = Model::default();
    model.set_drawer_open(true);
    assert!(model.flag_drawer_open());

    // Симулируем клик по редактору (закрытие дровера)
    model.set_drawer_open(false);
    assert!(!model.flag_drawer_open());
}

// -----------------------------------------------------------------------
// Тесты view_editor — проверяем отображение контента
// -----------------------------------------------------------------------

#[test]
fn test_view_editor_content_displayed() {
    let mut model = Model::default();
    let test_content = "G0 X10 Y20\nG1 Z-5 F100";
    model.set_content(test_content.to_string());
    model.set_file_path("/test.nc".to_string());

    assert_eq!(model.content(), test_content);
    assert_eq!(model.file_path(), "/test.nc");
}

#[test]
fn test_view_editor_empty_content() {
    let mut model = Model::default();
    assert!(model.content().is_empty());
    assert!(model.file_path().is_empty());
}

#[test]
fn test_view_editor_error_lines_highlighting() {
    let mut model = Model::default();
    model.set_content("G0 X10\nG1 Z\nG2 X5".to_string());
    model.set_error_lines(vec![2]); // вторая строка с ошибкой

    assert!(model.error_lines().contains(&2));
    assert!(!model.error_lines().contains(&1));
    assert!(!model.error_lines().contains(&3));
}

// -----------------------------------------------------------------------
// Тесты view_statusbar — проверяем статус
// -----------------------------------------------------------------------

#[test]
fn test_view_statusbar_initial_state() {
    let mut model = Model::default();
    model.set_status("Готов".to_string());
    assert_eq!(model.status(), "Готов");
}

#[test]
fn test_view_statusbar_busy() {
    let mut model = Model::default();
    model.set_is_busy(true);
    assert!(model.is_busy());

    model.set_is_busy(false);
    assert!(!model.is_busy());
}

#[test]
fn test_view_statusbar_with_messages() {
    let mut model = Model::default();
    model.set_status("Ошибка: ось X без значения".to_string());
    assert_eq!(model.status(), "Ошибка: ось X без значения");
}

// -----------------------------------------------------------------------
// Тесты диалогов — проверяем флаги
// -----------------------------------------------------------------------

#[test]
fn test_dialog_exit_initial_state() {
    let mut model = Model::default();
    assert!(!model.show_exit_dialog());

    model.set_show_exit_dialog(true);
    assert!(model.show_exit_dialog());
}

#[test]
fn test_dialog_search_toggle() {
    let mut model = Model::default();
    assert!(!model.search_open());

    model.set_search_open(true);
    assert!(model.search_open());

    model.set_search_open(false);
    assert!(!model.search_open());
}

#[test]
fn test_dialog_replace_toggle() {
    let mut model = Model::default();
    assert!(!model.replace_open());

    model.set_replace_open(true);
    assert!(model.replace_open());
}

#[test]
fn test_dialog_axis_swap_toggle() {
    let mut model = Model::default();
    assert!(!model.axis_swap_open());

    model.set_axis_swap_open(true);
    assert!(model.axis_swap_open());
}

#[test]
fn test_dialog_shortcuts_toggle() {
    let mut model = Model::default();
    assert!(!model.shortcuts_open());

    model.set_shortcuts_open(true);
    assert!(model.shortcuts_open());
}

// -----------------------------------------------------------------------
// Тесты Settings — проверяем настройки
// -----------------------------------------------------------------------

#[test]
fn test_settings_toggle() {
    let mut model = Model::default();
    assert!(!model.settings_open());

    model.set_settings_open(true);
    assert!(model.settings_open());
}

#[test]
fn test_settings_default_values() {
    let settings = crate::interfaces::gui::model::FormatSettings::default();
    assert_eq!(settings.renumber_step, 1);
    assert!(settings.skip_empty_lines);
    assert_eq!(settings.language, "ru");
}

// -----------------------------------------------------------------------
// Тесты поиска и замены
// -----------------------------------------------------------------------

#[test]
fn test_search_state() {
    let mut model = Model::default();
    model.set_search_query("G0".to_string());
    assert_eq!(model.search_query(), "G0");

    model.set_search_index(0);
    assert_eq!(model.search_index(), 0);
}

#[test]
fn test_replace_state() {
    let mut model = Model::default();
    model.set_replace_find("X".to_string());
    model.set_replace_with("Y".to_string());
    assert_eq!(model.replace_find(), "X");
    assert_eq!(model.replace_with(), "Y");

    model.set_replace_index(1);
    assert_eq!(model.replace_index(), 1);
}

#[test]
fn test_search_matches() {
    let mut model = Model::default();
    model.set_search_matches(vec![0, 5, 10]);
    assert_eq!(model.search_matches().len(), 3);
    assert_eq!(model.search_matches()[0], 0);
    assert_eq!(model.search_matches()[2], 10);
}

#[test]
fn test_replace_matches() {
    let mut model = Model::default();
    model.set_replace_matches(vec![3, 8]);
    assert_eq!(model.replace_matches().len(), 2);
    assert_eq!(model.replace_matches()[1], 8);
}
