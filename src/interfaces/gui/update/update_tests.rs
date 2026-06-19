use crate::interfaces::gui::intent::Intent;
use crate::interfaces::gui::model::{Model, PendingAction};
use crate::interfaces::gui::update::update::{invert_axes_by_letter, swap_axes};
use crate::shared::i18n;

/// Блокировка для глобального состояния i18n::LANG.
fn with_i18n_lock() -> std::sync::MutexGuard<'static, ()> {
    i18n::test_lock()
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

// -----------------------------------------------------------------------
// Поиск: DoSearch, FindNext
// -----------------------------------------------------------------------

#[test]
fn test_search_finds_occurrences() {
    let mut m = Model::default();
    m.set_content("G0 X10\nG0 Y20\nG1 Z30".to_string());
    m.set_search_query("G0".to_string());
    m.apply(&Intent::DoSearch);

    assert_eq!(m.search_matches().len(), 2);
    assert_eq!(m.search_index(), 0);
    assert_eq!(m.search_last_query(), "g0");
}

#[test]
fn test_search_case_insensitive() {
    let mut m = Model::default();
    m.set_content("G0 X10\ng0 Y20".to_string());
    m.set_search_query("g0".to_string());
    m.apply(&Intent::DoSearch);

    assert_eq!(m.search_matches().len(), 2);
}

#[test]
fn test_search_empty_query_clears() {
    let mut m = Model::default();
    m.set_content("G0 X10".to_string());
    m.set_search_query("G0".to_string());
    m.apply(&Intent::DoSearch);
    assert_eq!(m.search_matches().len(), 1);

    m.set_search_query(String::new());
    m.apply(&Intent::DoSearch);
    assert!(m.search_matches().is_empty());
    assert!(m.search_last_query().is_empty());
}

#[test]
fn test_find_next_cycles() {
    let mut m = Model::default();
    m.set_content("G0 X10\nG0 Y20\nG0 Z30".to_string());
    m.set_search_query("G0".to_string());
    m.apply(&Intent::DoSearch);
    assert_eq!(m.search_index(), 0);

    m.apply(&Intent::FindNext);
    assert_eq!(m.search_index(), 1);

    m.apply(&Intent::FindNext);
    assert_eq!(m.search_index(), 2);

    m.apply(&Intent::FindNext); // цикл
    assert_eq!(m.search_index(), 0);
}

#[test]
fn test_find_next_recalculates_on_query_change() {
    let mut m = Model::default();
    m.set_content("G0 X10\nG0 Y20\nM3 S1000".to_string());
    m.set_search_query("G0".to_string());
    m.apply(&Intent::DoSearch);
    assert_eq!(m.search_matches().len(), 2);
    m.apply(&Intent::FindNext);
    assert_eq!(m.search_index(), 1);

    // Меняем запрос — FindNext должен пересчитать
    m.set_search_query("M3".to_string());
    m.apply(&Intent::FindNext);
    assert_eq!(m.search_matches().len(), 1);
    assert_eq!(m.search_index(), 0);
}

// -----------------------------------------------------------------------
// Замена: ReplaceOne, ReplaceAll
// -----------------------------------------------------------------------

#[test]
fn test_replace_one_replaces_current() {
    let mut m = Model::default();
    m.set_content("G0 X10\nG0 Y20".to_string());
    m.set_replace_find("G0".to_string());
    m.set_replace_with("G1".to_string());
    // Сначала находим вхождения
    m.apply(&Intent::DoReplaceSearch);
    assert_eq!(m.replace_matches().len(), 2);
    assert_eq!(m.replace_index(), 0);

    // Заменяем первое
    m.apply(&Intent::ReplaceOne);
    assert_eq!(m.content(), "G1 X10\nG0 Y20");
    // После замены вхождения пересчитаны
    assert_eq!(m.replace_matches().len(), 1);
}

#[test]
fn test_replace_all_replaces_everything() {
    let mut m = Model::default();
    m.set_content("G0 X10\nG0 Y20\nG0 Z30".to_string());
    m.set_replace_find("G0".to_string());
    m.set_replace_with("G1".to_string());
    m.apply(&Intent::ReplaceAll);
    assert_eq!(m.content(), "G1 X10\nG1 Y20\nG1 Z30");
    assert!(m.replace_matches().is_empty());
}

#[test]
fn test_replace_case_insensitive() {
    let mut m = Model::default();
    m.set_content("g0 X10\nG0 Y20".to_string());
    m.set_replace_find("G0".to_string());
    m.set_replace_with("G1".to_string());
    m.apply(&Intent::ReplaceAll);
    assert_eq!(m.content(), "G1 X10\nG1 Y20");
}

#[test]
fn test_replace_find_next_cycles() {
    let mut m = Model::default();
    m.set_content("G0 X10\nG0 Y20\nG0 Z30".to_string());
    m.set_replace_find("G0".to_string());
    m.set_replace_with("G1".to_string());

    // Первый FindNext — пересчитывает
    m.apply(&Intent::ReplaceFindNext);
    assert_eq!(m.replace_matches().len(), 3);
    assert_eq!(m.replace_index(), 0);

    m.apply(&Intent::ReplaceFindNext);
    assert_eq!(m.replace_index(), 1);

    m.apply(&Intent::ReplaceFindNext);
    assert_eq!(m.replace_index(), 2);

    m.apply(&Intent::ReplaceFindNext);
    assert_eq!(m.replace_index(), 0);
}

// -----------------------------------------------------------------------
// Замена осей: swap_axes, invert_axes
// -----------------------------------------------------------------------

#[test]
fn test_swap_axes_z_x() {
    let input = "G0 X10 Z20\nG1 X-5 Z=71.304";
    let result = swap_axes(input, "Z", "X");
    assert_eq!(result, "G0 Z10 X20\nG1 Z-5 X=71.304");
}

#[test]
fn test_swap_axes_same_axis_noop() {
    let input = "G0 X10 Z20";
    let result = swap_axes(input, "X", "X");
    assert_eq!(result, input);
}

#[test]
fn test_invert_axes_simple() {
    // Простое число → инвертируется
    let input = "G0 X10 Y-20 X=71.304";
    let result = invert_axes_by_letter(input, "X");
    assert_eq!(result, "G0 X-10 Y-20 X=-71.304");
}

#[test]
fn test_invert_axes_expr() {
    // AxisExpr с выражением → оборачивается в -(expr)
    let input = "G0 Z=1+10 X=-20-10";
    let result = invert_axes_by_letter(input, "X");
    assert_eq!(result, "G0 Z=1+10 X=-(-20-10)");
}

#[test]
fn test_invert_axes_expr_positive() {
    // Положительное выражение без минуса
    let input = "G0 X=5*3/2";
    let result = invert_axes_by_letter(input, "X");
    assert_eq!(result, "G0 X=-(5*3/2)");
}

#[test]
fn test_invert_axes_expr_negative() {
    // Отрицательное выражение
    let input = "G0 X=-100+50";
    let result = invert_axes_by_letter(input, "X");
    assert_eq!(result, "G0 X=-(-100+50)");
}

#[test]
fn test_invert_axes_mixed_axes() {
    // Инвертируется только указанная ось
    let input = "G0 X10 Y-20 Z=71.304";
    let result = invert_axes_by_letter(input, "Z");
    assert_eq!(result, "G0 X10 Y-20 Z=-71.304");
}

#[test]
fn test_invert_axes_r_param() {
    // R-параметр: -R20 → R20
    let input = "G0 Z=-R20";
    let result = invert_axes_by_letter(input, "Z");
    assert_eq!(result, "G0 Z=R20");
}

#[test]
fn test_invert_axes_r_param_expr() {
    // Выражение с R-параметром: -R100+R1 → -(-R100+R1)
    let input = "G0 Z=-R100+R1";
    let result = invert_axes_by_letter(input, "Z");
    assert_eq!(result, "G0 Z=-(-R100+R1)");
}

#[test]
fn test_invert_axes_r_param_positive() {
    // R100+20 → -(R100+20)
    let input = "G0 Z=R100+20";
    let result = invert_axes_by_letter(input, "Z");
    assert_eq!(result, "G0 Z=-(R100+20)");
}

#[test]
fn test_invert_axes_no_axes() {
    let input = "G0 M3 S1000";
    let result = invert_axes_by_letter(input, "X");
    assert_eq!(result, "G0 M3 S1000");
}

#[test]
fn test_invert_axes_with_comments() {
    let input = "G0 X10 ; comment Z20";
    let result = invert_axes_by_letter(input, "X");
    assert_eq!(result, "G0 X-10 ; comment Z20");
}
