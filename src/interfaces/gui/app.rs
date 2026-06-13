//! App — точка входа eframe, связывает UI с data layer.

use std::sync::mpsc;
use std::time::Instant;

use eframe::egui;

use crate::data_layer::{
    DialogCommand, DialogEvent, EditorCommand, EditorEvent, FileCommand, FileEvent,
    PipelineCommand, PipelineEvent,
};
use crate::shared::i18n;

use super::intent::Intent;
use super::model::Model;
use super::view;

/// Главное приложение G-Code Editor.
pub struct GCodeApp {
    model: Model,
    cmd_tx: mpsc::Sender<EditorCommand>,
    evt_rx: mpsc::Receiver<EditorEvent>,
    /// Последнее время отправки TextChanged (для coalesce).
    last_text_change: Instant,
    /// Текст, ожидающий отправки (для coalesce).
    pending_text: Option<String>,
    /// Флаг: ожидание ответа от data layer на диалог.
    awaiting_picker: bool,
    awaiting_save_picker: bool,
    awaiting_dialog: bool,
}

impl GCodeApp {
    pub fn new(cmd_tx: mpsc::Sender<EditorCommand>, evt_rx: mpsc::Receiver<EditorEvent>) -> Self {
        let mut model = Model {
            status: i18n::locale().status.ready.to_string(),
            ..Default::default()
        };
        model.load_settings();
        i18n::set_lang(&model.format_settings.language);
        Self {
            model,
            cmd_tx,
            evt_rx,
            last_text_change: Instant::now(),
            pending_text: None,
            awaiting_picker: false,
            awaiting_save_picker: false,
            awaiting_dialog: false,
        }
    }

    /// Отправляет TextChanged с coalesce.
    #[allow(dead_code)]
    fn send_text_changed(&mut self, content: &str) {
        self.pending_text = Some(content.to_string());
        self.last_text_change = Instant::now();
    }

    fn flush_pending_text(&mut self) {
        if let Some(text) = self.pending_text.take() {
            let _ = self
                .cmd_tx
                .send(EditorCommand::Pipeline(PipelineCommand::TextChanged(text)));
        }
    }

    /// Обрабатывает событие от data layer: мутирует модель.
    /// Вынесено из update() для тестирования.
    pub fn handle_event(&mut self, event: EditorEvent) {
        match event {
            EditorEvent::Pipeline(pe) => match pe {
                PipelineEvent::Formatted { content, errors } => {
                    if !content.is_empty() {
                        self.model.content = content;
                    }
                    if errors.is_empty() {
                        self.model.status = i18n::locale().status.formatted.to_string();
                    } else {
                        let first = &errors[0];
                        self.model.status =
                            i18n::fmt_errors_found(errors.len(), &first.message, first.line);
                    }
                    self.model.is_busy = false;
                }
                PipelineEvent::Validated { errors } => {
                    if errors.is_empty() {
                        self.model.status = i18n::locale().status.no_errors.to_string();
                    } else {
                        let first = &errors[0];
                        self.model.status =
                            i18n::fmt_errors_found(errors.len(), &first.message, first.line);
                    }
                    self.model.is_busy = false;
                }
            },
            EditorEvent::File(fe) => match fe {
                FileEvent::Loaded { content, file_path } => {
                    self.model.content = content;
                    self.model.file_path = file_path;
                    self.model.modified = false;
                    self.model.is_busy = false;
                    self.model.status = i18n::locale().status.file_opened.to_string();
                }
                FileEvent::Saved { file_path } => {
                    self.model.file_path = file_path;
                    self.model.modified = false;
                    self.model.is_busy = false;
                    match self.model.save_and_exec.take() {
                        Some(super::model::PendingAction::Exit) => {
                            std::process::exit(0);
                        }
                        Some(super::model::PendingAction::CloseFile) => {
                            self.model.content.clear();
                            self.model.file_path.clear();
                            self.model.modified = false;
                            self.model.status = i18n::locale().status.file_closed.to_string();
                        }
                        Some(super::model::PendingAction::OpenNewFile) => {
                            self.model.is_busy = true;
                            let _ = self.cmd_tx.send(EditorCommand::File(FileCommand::OpenFile));
                        }
                        None => {
                            self.model.status = i18n::locale().status.saved.to_string();
                        }
                    }
                }
            },
            EditorEvent::Dialog(de) => match de {
                DialogEvent::RequestFilePicker { mode } => match mode {
                    crate::data_layer::FilePickerMode::Open => {
                        self.awaiting_picker = true;
                    }
                    crate::data_layer::FilePickerMode::Save => {
                        self.awaiting_save_picker = true;
                    }
                },
                DialogEvent::RequestDialog {
                    title: _,
                    message: _,
                } => {
                    self.awaiting_dialog = true;
                }
                DialogEvent::NotifyUser { message, level: _ } => {
                    self.model.status = message;
                }
                DialogEvent::Idle => {
                    self.model.is_busy = false;
                }
            },
        }
    }

    /// Обрабатывает намерение пользователя: отправляет команды в data layer
    /// или мутирует модель.
    /// Вынесено из update() для тестирования.
    pub fn handle_intent(&mut self, intent: &Intent) {
        match intent {
            Intent::Format => {
                if self.model.content.is_empty() {
                    self.model.status = i18n::locale().status.empty_editor.to_string();
                } else {
                    self.model.status = i18n::locale().status.formatting.to_string();
                    self.model.is_busy = true;
                    let settings = self.model.format_settings.clone();
                    let content = self.model.content.clone();
                    let _ = self
                        .cmd_tx
                        .send(EditorCommand::Pipeline(PipelineCommand::Format {
                            content,
                            renumber_step: settings.renumber_step,
                            skip_empty_lines: settings.skip_empty_lines,
                        }));
                }
            }
            Intent::Validate => {
                if self.model.content.is_empty() {
                    self.model.status = i18n::locale().status.empty_validate.to_string();
                } else {
                    self.model.status = i18n::locale().status.validating.to_string();
                    self.model.is_busy = true;
                    let _ = self
                        .cmd_tx
                        .send(EditorCommand::Pipeline(PipelineCommand::Validate(
                            self.model.content.clone(),
                        )));
                }
            }
            Intent::OpenFile => {
                if self.model.modified && !self.model.file_path.is_empty() {
                    self.model.show_exit_dialog = true;
                    self.model.pending_action = Some(super::model::PendingAction::OpenNewFile);
                } else {
                    self.model.is_busy = true;
                    let _ = self.cmd_tx.send(EditorCommand::File(FileCommand::OpenFile));
                }
            }
            Intent::SaveFile => {
                if self.model.file_path.is_empty() {
                    self.model.is_busy = true;
                    let _ = self.cmd_tx.send(EditorCommand::File(FileCommand::SaveFile {
                        path: None,
                        content: self.model.content.clone(),
                    }));
                } else {
                    let path = self.model.file_path.clone();
                    self.model.is_busy = true;
                    let _ = self.cmd_tx.send(EditorCommand::File(FileCommand::SaveFile {
                        path: Some(path),
                        content: self.model.content.clone(),
                    }));
                }
            }
            Intent::SaveAs => {
                self.model.is_busy = true;
                let _ = self.cmd_tx.send(EditorCommand::File(FileCommand::SaveFile {
                    path: None,
                    content: self.model.content.clone(),
                }));
            }
            Intent::ConfirmSave => {
                self.model.show_exit_dialog = false;
                self.model.save_and_exec = self.model.pending_action.take();
                let path = if self.model.file_path.is_empty() {
                    None
                } else {
                    Some(self.model.file_path.clone())
                };
                let content = std::mem::take(&mut self.model.content);
                let _ = self
                    .cmd_tx
                    .send(EditorCommand::File(FileCommand::SaveFile { path, content }));
            }
            Intent::DiscardAndContinue => {
                self.model.show_exit_dialog = false;
                self.model.modified = false;
                let action = self.model.pending_action.take();
                match action {
                    Some(super::model::PendingAction::Exit) => std::process::exit(0),
                    Some(super::model::PendingAction::CloseFile) => {
                        self.model.content.clear();
                        self.model.file_path.clear();
                        self.model.status = i18n::locale().status.file_closed.to_string();
                    }
                    Some(super::model::PendingAction::OpenNewFile) => {
                        self.model.is_busy = true;
                        let _ = self.cmd_tx.send(EditorCommand::File(FileCommand::OpenFile));
                    }
                    None => {}
                }
            }
            _ => {
                self.model.apply(intent);
            }
        }
    }
}

impl eframe::App for GCodeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let is_busy = self.model.is_busy;

        // === Проверка на закрытие окна ===
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.model.modified && !self.model.file_path.is_empty() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.model.show_exit_dialog = true;
                self.model.pending_action = Some(super::model::PendingAction::Exit);
            }
        }

        // === Получение событий от data layer ===
        while let Ok(event) = self.evt_rx.try_recv() {
            self.handle_event(event);
        }

        // === Горячие клавиши ===
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F5)) {
            self.handle_intent(&Intent::Format);
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F6)) {
            self.handle_intent(&Intent::Validate);
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::O)) {
            self.handle_intent(&Intent::OpenFile);
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::S)) {
            let shift = ctx.input(|i| i.modifiers.shift);
            if shift {
                self.handle_intent(&Intent::SaveAs);
            } else {
                self.handle_intent(&Intent::SaveFile);
            }
        }

        // === Coalesce: отправляем TextChanged, если прошло 100 мс с последнего изменения ===
        if self.pending_text.is_some()
            && self.last_text_change.elapsed() >= std::time::Duration::from_millis(100)
        {
            self.flush_pending_text();
        }

        // 1. View → Intent: собираем намерения от UI
        let mut all_intents = view::collect_intents(ctx, is_busy, &self.model);
        all_intents.extend(view::view_settings(&self.model, ctx));
        all_intents.extend(view::view_exit_dialog(&self.model, ctx));
        all_intents.extend(view::view_shortcuts(&self.model, ctx));

        // 2. Intent — обрабатываем через handle_intent
        for intent in &all_intents {
            self.handle_intent(intent);
        }

        // 3. View: статусбар и редактор
        view::view_statusbar(&self.model, ctx);
        view::view_editor(
            &mut self.model,
            ctx,
            &self.cmd_tx,
            &mut self.last_text_change,
            &mut self.pending_text,
        );

        // 4. Если data layer запрашивает file picker — показываем его
        if self.awaiting_picker {
            self.awaiting_picker = false;
            let file = rfd::FileDialog::new().pick_file();
            let _ = self
                .cmd_tx
                .send(EditorCommand::Dialog(DialogCommand::FilePickerResult {
                    path: file.map(|p| p.to_string_lossy().to_string()),
                    mode: crate::data_layer::FilePickerMode::Open,
                }));
        }
        if self.awaiting_save_picker {
            self.awaiting_save_picker = false;
            let file = rfd::FileDialog::new().save_file();
            let _ = self
                .cmd_tx
                .send(EditorCommand::Dialog(DialogCommand::FilePickerResult {
                    path: file.map(|p| p.to_string_lossy().to_string()),
                    mode: crate::data_layer::FilePickerMode::Save,
                }));
        }

        // Repaint для спиннера и coalesce
        if self.model.is_busy || self.pending_text.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
    }
}

#[cfg(test)]
mod tests {
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
}
