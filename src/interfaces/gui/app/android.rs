//! App для Android — точка входа android-activity.
//! Содержит GCodeApp без eframe, с ручным циклом событий.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::data_layer::{
    DialogEvent, EditorCommand, EditorEvent, FileEvent, PipelineCommand, PipelineEvent,
};
use crate::interfaces::gui::intent::Intent;
use crate::interfaces::gui::model::Model;
use crate::shared::i18n;

/// GCodeApp для Android — без eframe, с ручным управлением циклом событий.
pub struct GCodeApp {
    pub model: Model,
    cmd_tx: mpsc::Sender<EditorCommand>,
    evt_rx: mpsc::Receiver<EditorEvent>,
    last_text_change: Instant,
    pending_text: Option<String>,
}

impl GCodeApp {
    pub fn new(cmd_tx: mpsc::Sender<EditorCommand>, evt_rx: mpsc::Receiver<EditorEvent>) -> Self {
        Self {
            model: Model::default(),
            cmd_tx,
            evt_rx,
            last_text_change: Instant::now(),
            pending_text: None,
        }
    }

    /// Применяет намерение к модели.
    pub fn handle_intent(&mut self, intent: &Intent) {
        self.model.apply(intent);
    }

    /// Отправляет команду в data layer.
    pub fn send_command(&self, cmd: EditorCommand) {
        let _ = self.cmd_tx.send(cmd);
    }

    /// Отправляет TextChanged с дебаунсом 100 мс.
    pub fn send_text_changed(&mut self, content: &str) {
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

    /// Обрабатывает события от data layer и применяет их к модели.
    pub fn handle_event(&mut self, event: EditorEvent) {
        match event {
            EditorEvent::Pipeline(pe) => match pe {
                PipelineEvent::Formatted { content, errors } => {
                    if !content.is_empty() {
                        self.model.set_content(content);
                        self.model.set_modified(true);
                    }
                    self.model.set_error_lines(
                        errors.iter().map(|e| e.line).filter(|l| *l > 0).collect(),
                    );
                    if errors.is_empty() {
                        self.model
                            .set_status(i18n::locale().status.formatted.to_string());
                    } else {
                        let first = &errors[0];
                        self.model.set_status(i18n::fmt_errors_found(
                            errors.len(),
                            &first.message,
                            first.line,
                        ));
                    }
                    self.model.set_is_busy(false);
                }
                PipelineEvent::Validated { errors } => {
                    self.model.set_error_lines(
                        errors.iter().map(|e| e.line).filter(|l| *l > 0).collect(),
                    );
                    if errors.is_empty() {
                        self.model
                            .set_status(i18n::locale().status.no_errors.to_string());
                    } else {
                        let first = &errors[0];
                        self.model.set_status(i18n::fmt_errors_found(
                            errors.len(),
                            &first.message,
                            first.line,
                        ));
                    }
                    self.model.set_is_busy(false);
                }
            },
            EditorEvent::File(fe) => match fe {
                FileEvent::Loaded { content, file_path } => {
                    self.model.set_content(content);
                    self.model.set_file_path(file_path);
                    self.model.set_modified(false);
                    self.model.set_is_busy(false);
                    self.model
                        .set_status(i18n::locale().status.file_opened.to_string());
                    self.model.set_error_lines(Vec::new());
                }
                FileEvent::Saved { file_path } => {
                    self.model.set_file_path(file_path);
                    self.model.set_modified(false);
                    self.model.set_is_busy(false);
                    self.model
                        .set_status(i18n::locale().status.saved.to_string());
                }
            },
            EditorEvent::Dialog(de) => match de {
                DialogEvent::RequestFilePicker { mode: _ } => {
                    // Android пока не поддерживает file picker — заглушка
                }
                DialogEvent::RequestDialog {
                    title: _,
                    message: _,
                } => {
                    // Android пока не поддерживает диалоги — заглушка
                }
                DialogEvent::NotifyUser { message, level: _ } => {
                    self.model.set_status(message);
                }
                DialogEvent::Idle => {
                    self.model.set_is_busy(false);
                }
            },
        }
    }

    /// Дренирует канал событий и применяет их к модели.
    /// Возвращает true, если были обработаны какие-либо события.
    pub fn poll_events(&mut self) -> bool {
        let mut had_events = false;
        while let Ok(event) = self.evt_rx.try_recv() {
            self.handle_event(event);
            had_events = true;
        }

        // Flush TextChanged с дебаунсом
        if self.pending_text.is_some()
            && self.last_text_change.elapsed() >= Duration::from_millis(100)
        {
            self.flush_pending_text();
        }

        had_events
    }
}
