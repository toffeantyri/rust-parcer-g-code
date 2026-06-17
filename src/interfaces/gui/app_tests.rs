use super::*;
use crate::interfaces::gui::model::PendingAction;

fn make_app() -> GCodeApp {
    let (tx, _) = mpsc::channel();
    let (_evt_tx, evt_rx) = mpsc::channel();
    GCodeApp::new(tx, evt_rx)
}

fn make_app_with_content(content: &str, file_path: &str) -> GCodeApp {
    let (tx, _) = mpsc::channel();
    let (_evt_tx, evt_rx) = mpsc::channel();
    let mut app = GCodeApp::new(tx, evt_rx);
    app.model.content = content.to_string();
    app.model.file_path = file_path.to_string();
    app.model.modified = true;
    app
}

// -----------------------------------------------------------------------
// handle_event — Pipeline
// -----------------------------------------------------------------------

#[test]
fn test_handle_formatted_updates_content() {
    let mut app = make_app();
    app.model.content = "old".to_string();

    app.handle_event(EditorEvent::Pipeline(PipelineEvent::Formatted {
        content: "G0 X10".to_string(),
        errors: vec![],
    }));

    assert_eq!(app.model.content, "G0 X10");
    assert_eq!(
        app.model.status,
        i18n::locale().status.formatted.to_string()
    );
    assert!(!app.model.is_busy);
}

#[test]
fn test_handle_formatted_empty_content_skips() {
    let mut app = make_app();
    app.model.content = "keep me".to_string();

    app.handle_event(EditorEvent::Pipeline(PipelineEvent::Formatted {
        content: "".to_string(),
        errors: vec![],
    }));

    // Пустой content не должен затирать существующий
    assert_eq!(app.model.content, "keep me");
}

#[test]
fn test_handle_formatted_with_errors() {
    let mut app = make_app();
    let err = crate::shared::ValidationMessage::error(3, "ось X без значения");

    app.handle_event(EditorEvent::Pipeline(PipelineEvent::Formatted {
        content: "G0 X".to_string(),
        errors: vec![err],
    }));

    assert!(app.model.status.contains("1"));
    assert!(app.model.status.contains("ось X"));
    assert!(app.model.status.contains("3"));
    assert!(!app.model.is_busy);
}

#[test]
fn test_handle_validated_no_errors() {
    let mut app = make_app();
    app.model.is_busy = true;

    app.handle_event(EditorEvent::Pipeline(PipelineEvent::Validated {
        errors: vec![],
    }));

    assert_eq!(
        app.model.status,
        i18n::locale().status.no_errors.to_string()
    );
    assert!(!app.model.is_busy);
}

#[test]
fn test_handle_validated_with_errors() {
    let mut app = make_app();
    let err = crate::shared::ValidationMessage::error(5, "ошибка");

    app.handle_event(EditorEvent::Pipeline(PipelineEvent::Validated {
        errors: vec![err],
    }));

    assert!(app.model.status.contains("5"));
}

// -----------------------------------------------------------------------
// handle_event — File
// -----------------------------------------------------------------------

#[test]
fn test_handle_loaded() {
    let mut app = make_app();

    app.handle_event(EditorEvent::File(FileEvent::Loaded {
        content: "G0 X10".to_string(),
        file_path: "/path/file.nc".to_string(),
    }));

    assert_eq!(app.model.content, "G0 X10");
    assert_eq!(app.model.file_path, "/path/file.nc");
    assert!(!app.model.modified);
    assert!(!app.model.is_busy);
}

#[test]
fn test_handle_saved_without_pending_action() {
    let mut app = make_app();
    app.model.is_busy = true;

    app.handle_event(EditorEvent::File(FileEvent::Saved {
        file_path: "/path/file.nc".to_string(),
    }));

    assert_eq!(app.model.file_path, "/path/file.nc");
    assert!(!app.model.modified);
    assert!(!app.model.is_busy);
    assert_eq!(app.model.status, i18n::locale().status.saved.to_string());
}

#[test]
fn test_handle_saved_with_close_file_action() {
    let mut app = make_app_with_content("G0 X10", "/path/file.nc");
    app.model.save_and_exec = Some(PendingAction::CloseFile);

    app.handle_event(EditorEvent::File(FileEvent::Saved {
        file_path: "/path/file.nc".to_string(),
    }));

    assert!(app.model.content.is_empty());
    assert!(app.model.file_path.is_empty());
    assert!(!app.model.modified);
    assert_eq!(
        app.model.status,
        i18n::locale().status.file_closed.to_string()
    );
}

// -----------------------------------------------------------------------
// handle_event — Dialog
// -----------------------------------------------------------------------

#[test]
fn test_handle_request_file_picker_open() {
    let mut app = make_app();
    app.handle_event(EditorEvent::Dialog(DialogEvent::RequestFilePicker {
        mode: crate::data_layer::FilePickerMode::Open,
    }));
    assert!(app.awaiting_picker);
    assert!(!app.awaiting_save_picker);
}

#[test]
fn test_handle_request_file_picker_save() {
    let mut app = make_app();
    app.handle_event(EditorEvent::Dialog(DialogEvent::RequestFilePicker {
        mode: crate::data_layer::FilePickerMode::Save,
    }));
    assert!(app.awaiting_save_picker);
    assert!(!app.awaiting_picker);
}

#[test]
fn test_handle_notify_user() {
    let mut app = make_app();
    app.handle_event(EditorEvent::Dialog(DialogEvent::NotifyUser {
        message: "тест".to_string(),
        level: crate::data_layer::NotifyLevel::Info,
    }));
    assert_eq!(app.model.status, "тест");
}

#[test]
fn test_handle_idle() {
    let mut app = make_app();
    app.model.is_busy = true;
    app.handle_event(EditorEvent::Dialog(DialogEvent::Idle));
    assert!(!app.model.is_busy);
}

// -----------------------------------------------------------------------
// handle_intent — Format / Validate
// -----------------------------------------------------------------------

#[test]
fn test_intent_format_empty_editor() {
    let mut app = make_app();
    app.model.content = "".to_string();
    app.handle_intent(&Intent::Format);
    assert_eq!(
        app.model.status,
        i18n::locale().status.empty_editor.to_string()
    );
    assert!(!app.model.is_busy);
}

#[test]
fn test_intent_format_with_content_sets_busy() {
    let mut app = make_app_with_content("G0 X10", "");
    app.handle_intent(&Intent::Format);
    assert!(app.model.is_busy);
    assert_eq!(
        app.model.status,
        i18n::locale().status.formatting.to_string()
    );
}

#[test]
fn test_intent_validate_empty_editor() {
    let mut app = make_app();
    app.model.content = "".to_string();
    app.handle_intent(&Intent::Validate);
    assert_eq!(
        app.model.status,
        i18n::locale().status.empty_validate.to_string()
    );
}

#[test]
fn test_intent_validate_with_content_sets_busy() {
    let mut app = make_app_with_content("G0 X10", "");
    app.handle_intent(&Intent::Validate);
    assert!(app.model.is_busy);
}

// -----------------------------------------------------------------------
// handle_intent — File
// -----------------------------------------------------------------------

#[test]
fn test_intent_open_file_modified_shows_dialog() {
    let mut app = make_app_with_content("G0 X10", "/path/file.nc");
    app.handle_intent(&Intent::OpenFile);
    assert!(app.model.show_exit_dialog);
    assert_eq!(app.model.pending_action, Some(PendingAction::OpenNewFile));
}

#[test]
fn test_intent_open_file_not_modified_sends_command() {
    let mut app = make_app();
    app.handle_intent(&Intent::OpenFile);
    assert!(app.model.is_busy);
}

#[test]
fn test_intent_save_file_with_path() {
    let mut app = make_app_with_content("G0 X10", "/path/file.nc");
    app.handle_intent(&Intent::SaveFile);
    assert!(app.model.is_busy);
}

#[test]
fn test_intent_save_file_without_path() {
    let mut app = make_app_with_content("G0 X10", "");
    app.handle_intent(&Intent::SaveFile);
    assert!(app.model.is_busy);
}

#[test]
fn test_intent_save_as() {
    let mut app = make_app_with_content("G0 X10", "/path/file.nc");
    app.handle_intent(&Intent::SaveAs);
    assert!(app.model.is_busy);
}

// -----------------------------------------------------------------------
// handle_intent — ConfirmSave / DiscardAndContinue
// -----------------------------------------------------------------------

#[test]
fn test_intent_confirm_save_sets_save_and_exec() {
    let mut app = make_app_with_content("G0 X10", "/path/file.nc");
    app.model.pending_action = Some(PendingAction::Exit);
    app.handle_intent(&Intent::ConfirmSave);
    assert_eq!(app.model.save_and_exec, Some(PendingAction::Exit));
    assert!(!app.model.show_exit_dialog);
}

#[test]
fn test_intent_discard_close_file() {
    let mut app = make_app_with_content("G0 X10", "/path/file.nc");
    app.model.pending_action = Some(PendingAction::CloseFile);
    app.handle_intent(&Intent::DiscardAndContinue);
    assert!(app.model.content.is_empty());
    assert!(app.model.file_path.is_empty());
    assert_eq!(
        app.model.status,
        i18n::locale().status.file_closed.to_string()
    );
}

#[test]
fn test_intent_discard_open_new_file() {
    let mut app = make_app_with_content("G0 X10", "/path/file.nc");
    app.model.pending_action = Some(PendingAction::OpenNewFile);
    app.handle_intent(&Intent::DiscardAndContinue);
    assert!(app.model.is_busy);
    assert!(!app.model.show_exit_dialog);
}

#[test]
fn test_intent_toggle_shortcuts() {
    let mut app = make_app();
    assert!(!app.model.shortcuts_open);
    app.handle_intent(&Intent::ToggleShortcuts);
    assert!(app.model.shortcuts_open);
    app.handle_intent(&Intent::ToggleShortcuts);
    assert!(!app.model.shortcuts_open);
}
