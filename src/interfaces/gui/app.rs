//! App — точка входа eframe, связывает UI с data layer.

use std::sync::mpsc;
use std::time::Instant;

use eframe::egui;

use crate::data_layer::{EditorCommand, EditorEvent};

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
            status: "Готов к работе. Откройте файл G-кода.".to_string(),
            ..Default::default()
        };
        model.load_settings();
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
            let _ = self.cmd_tx.send(EditorCommand::TextChanged(text));
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
            match event {
                EditorEvent::Formatted {
                    content,
                    errors,
                    file_path,
                } => {
                    if !content.is_empty() {
                        self.model.content = content;
                    }
                    // Если закрываем файл — не восстанавливаем file_path из ответа data layer
                    if self.model.closing_after_save {
                        self.model.closing_after_save = false;
                    } else if let Some(path) = file_path {
                        self.model.file_path = path;
                        self.model.modified = false;
                    }
                    if errors.is_empty() {
                        self.model.status = "Форматирование завершено".to_string();
                    } else {
                        let first = &errors[0];
                        self.model.status = format!(
                            "Найдено {} ошибок. Первая: {} [строка {}]",
                            errors.len(),
                            first.message,
                            first.line,
                        );
                    }
                    self.model.is_busy = false;
                    // Если выходим — завершаем процесс после ответа data layer
                    if self.model.exiting_after_save {
                        std::process::exit(0);
                    }
                }
                EditorEvent::Validated { errors } => {
                    if errors.is_empty() {
                        self.model.status = "Ошибок не найдено".to_string();
                    } else {
                        let first = &errors[0];
                        self.model.status = format!(
                            "Найдено {} ошибок. Первая: {} [строка {}]",
                            errors.len(),
                            first.message,
                            first.line,
                        );
                    }
                    self.model.is_busy = false;
                }
                EditorEvent::RequestFilePicker { mode } => {
                    match mode {
                        crate::data_layer::FilePickerMode::Open => {
                            self.awaiting_picker = true;
                        }
                        crate::data_layer::FilePickerMode::Save => {
                            self.awaiting_save_picker = true;
                        }
                    }
                    self.model.is_busy = false;
                }
                EditorEvent::RequestDialog {
                    title: _,
                    message: _,
                } => {
                    self.awaiting_dialog = true;
                    self.model.is_busy = false;
                }
                EditorEvent::NotifyUser { message, level: _ } => {
                    self.model.status = message;
                }
                EditorEvent::Idle => {
                    self.model.is_busy = false;
                }
            }
        }

        // === Coalesce: отправляем TextChanged, если прошло 100 мс с последнего изменения ===
        if self.pending_text.is_some()
            && self.last_text_change.elapsed() >= std::time::Duration::from_millis(100)
        {
            self.flush_pending_text();
        }

        // 1. View → Intent: собираем намерения от UI
        let mut all_intents = view::collect_intents(ctx, is_busy, &self.model.file_path);
        all_intents.extend(view::view_settings(&self.model, ctx));
        all_intents.extend(view::view_exit_dialog(&self.model, ctx));

        // 2. Intent — разделяем: data layer команды отправляем в канал,
        //    остальные — в model.apply()
        for intent in &all_intents {
            match intent {
                super::intent::Intent::Format => {
                    if self.model.content.is_empty() {
                        self.model.status = "Редактор пуст. Нечего форматировать.".to_string();
                    } else {
                        self.model.status = "Форматирование...".to_string();
                        self.model.is_busy = true;
                        let settings = self.model.format_settings.clone();
                        let content = self.model.content.clone();
                        let _ = self.cmd_tx.send(EditorCommand::Format {
                            content,
                            renumber_step: settings.renumber_step,
                            skip_empty_lines: settings.skip_empty_lines,
                        });
                    }
                }
                super::intent::Intent::Validate => {
                    if self.model.content.is_empty() {
                        self.model.status = "Редактор пуст. Нечего проверять.".to_string();
                    } else {
                        self.model.status = "Проверка...".to_string();
                        self.model.is_busy = true;
                        let _ = self
                            .cmd_tx
                            .send(EditorCommand::Validate(self.model.content.clone()));
                    }
                }
                super::intent::Intent::OpenFile => {
                    if self.model.modified && !self.model.file_path.is_empty() {
                        self.model.show_exit_dialog = true;
                        self.model.pending_action = Some(super::model::PendingAction::OpenNewFile);
                    } else {
                        self.model.is_busy = true;
                        let _ = self.cmd_tx.send(EditorCommand::OpenFile);
                    }
                }
                super::intent::Intent::SaveFile => {
                    if self.model.file_path.is_empty() {
                        self.model.is_busy = true;
                        let _ = self.cmd_tx.send(EditorCommand::SaveFile {
                            path: None,
                            content: self.model.content.clone(),
                        });
                    } else {
                        let path = self.model.file_path.clone();
                        self.model.is_busy = true;
                        let _ = self.cmd_tx.send(EditorCommand::SaveFile {
                            path: Some(path),
                            content: self.model.content.clone(),
                        });
                    }
                }
                super::intent::Intent::SaveAs => {
                    self.model.is_busy = true;
                    let _ = self.cmd_tx.send(EditorCommand::SaveFile {
                        path: None,
                        content: self.model.content.clone(),
                    });
                }
                super::intent::Intent::ConfirmSave => {
                    self.model.show_exit_dialog = false;
                    let action = self.model.pending_action.take();
                    match action {
                        Some(super::model::PendingAction::Exit) => {
                            // Сохраняем и выходим
                            self.model.exiting_after_save = true;
                            if self.model.file_path.is_empty() {
                                self.model.is_busy = true;
                                let _ = self.cmd_tx.send(EditorCommand::SaveFile {
                                    path: None,
                                    content: std::mem::take(&mut self.model.content),
                                });
                            } else {
                                let path = self.model.file_path.clone();
                                let content = std::mem::take(&mut self.model.content);
                                let _ = self.cmd_tx.send(EditorCommand::SaveFile {
                                    path: Some(path),
                                    content,
                                });
                            }
                        }
                        Some(super::model::PendingAction::CloseFile) => {
                            self.model.closing_after_save = true;
                            if self.model.file_path.is_empty() {
                                let content = std::mem::take(&mut self.model.content);
                                let _ = self.cmd_tx.send(EditorCommand::SaveFile {
                                    path: None,
                                    content,
                                });
                            } else {
                                let path = self.model.file_path.clone();
                                let content = std::mem::take(&mut self.model.content);
                                let _ = self.cmd_tx.send(EditorCommand::SaveFile {
                                    path: Some(path),
                                    content,
                                });
                            }
                            self.model.content.clear();
                            self.model.file_path.clear();
                            self.model.modified = false;
                            self.model.status = "Файл закрыт.".to_string();
                        }
                        Some(super::model::PendingAction::OpenNewFile) => {
                            if self.model.file_path.is_empty() {
                                let content = std::mem::take(&mut self.model.content);
                                let _ = self.cmd_tx.send(EditorCommand::SaveFile {
                                    path: None,
                                    content,
                                });
                            } else {
                                let path = self.model.file_path.clone();
                                let content = std::mem::take(&mut self.model.content);
                                let _ = self.cmd_tx.send(EditorCommand::SaveFile {
                                    path: Some(path),
                                    content,
                                });
                            }
                            self.model.is_busy = true;
                            let _ = self.cmd_tx.send(EditorCommand::OpenFile);
                        }
                        None => {}
                    }
                }
                super::intent::Intent::DiscardAndContinue => {
                    self.model.show_exit_dialog = false;
                    self.model.modified = false;
                    let action = self.model.pending_action.take();
                    match action {
                        Some(super::model::PendingAction::Exit) => std::process::exit(0),
                        Some(super::model::PendingAction::CloseFile) => {
                            self.model.content.clear();
                            self.model.file_path.clear();
                            self.model.status = "Файл закрыт.".to_string();
                        }
                        Some(super::model::PendingAction::OpenNewFile) => {
                            self.model.is_busy = true;
                            let _ = self.cmd_tx.send(EditorCommand::OpenFile);
                        }
                        None => {}
                    }
                }
                _ => {
                    self.model.apply(intent);
                }
            }
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
            let _ = self.cmd_tx.send(EditorCommand::FilePickerResult {
                path: file.map(|p| p.to_string_lossy().to_string()),
                mode: crate::data_layer::FilePickerMode::Open,
            });
        }
        if self.awaiting_save_picker {
            self.awaiting_save_picker = false;
            let file = rfd::FileDialog::new().save_file();
            let _ = self.cmd_tx.send(EditorCommand::FilePickerResult {
                path: file.map(|p| p.to_string_lossy().to_string()),
                mode: crate::data_layer::FilePickerMode::Save,
            });
        }

        // Repaint для спиннера и coalesce
        if self.model.is_busy || self.pending_text.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
    }
}
