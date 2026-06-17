//! App — точка входа eframe, связывает UI с data layer.

use std::sync::mpsc;
use std::time::Instant;

use egui;

use crate::data_layer::{
    DialogCommand, DialogEvent, EditorCommand, EditorEvent, FileCommand, FileEvent,
    PipelineCommand, PipelineEvent,
};
use crate::shared::i18n;

use crate::interfaces::gui::intent::Intent;
use crate::interfaces::gui::model::{Model, PendingAction};
use crate::interfaces::gui::view;

/// Главное приложение G-Code Editor.
pub struct GCodeApp {
    pub(crate) model: Model,
    cmd_tx: mpsc::Sender<EditorCommand>,
    evt_rx: mpsc::Receiver<EditorEvent>,
    /// Последнее время отправки TextChanged (для coalesce).
    last_text_change: Instant,
    /// Текст, ожидающий отправки (для coalesce).
    pending_text: Option<String>,
    /// Флаг: ожидание ответа от data layer на диалог.
    pub(crate) awaiting_picker: bool,
    pub(crate) awaiting_save_picker: bool,
    pub(crate) awaiting_dialog: bool,
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
                        self.model.modified = true;
                    }
                    // Собираем номера строк с ошибками для подсветки
                    self.model.error_lines =
                        errors.iter().map(|e| e.line).filter(|l| *l > 0).collect();
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
                    // Собираем номера строк с ошибками для подсветки
                    self.model.error_lines =
                        errors.iter().map(|e| e.line).filter(|l| *l > 0).collect();
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
                    self.model.error_lines.clear(); // при загрузке нового файла ошибок нет
                }
                FileEvent::Saved { file_path } => {
                    self.model.file_path = file_path;
                    self.model.modified = false;
                    self.model.is_busy = false;
                    match self.model.save_and_exec.take() {
                        Some(PendingAction::Exit) => {
                            std::process::exit(0);
                        }
                        Some(PendingAction::CloseFile) => {
                            self.model.content.clear();
                            self.model.file_path.clear();
                            self.model.modified = false;
                            self.model.status = i18n::locale().status.file_closed.to_string();
                        }
                        Some(PendingAction::OpenNewFile) => {
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
                    self.model.pending_action = Some(PendingAction::OpenNewFile);
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
                    Some(PendingAction::Exit) => std::process::exit(0),
                    Some(PendingAction::CloseFile) => {
                        self.model.content.clear();
                        self.model.file_path.clear();
                        self.model.status = i18n::locale().status.file_closed.to_string();
                    }
                    Some(PendingAction::OpenNewFile) => {
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
                self.model.pending_action = Some(PendingAction::Exit);
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

        // Если текст изменился после view_editor — очищаем подсветку ошибок
        if self.pending_text.is_some() {
            self.model.error_lines.clear();
        }

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
#[path = "app_tests.rs"]
mod tests;
