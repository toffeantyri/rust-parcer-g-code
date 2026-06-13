//! Data layer — отдельный поток для ресурсоёмких операций (pipeline, IO).
//!
//! Архитектура: два mpsc-канала между UI потоком и data thread.
//!   UI ──EditorCommand──→ Data Thread
//!   UI ←──EditorEvent──── Data Thread

use std::sync::mpsc;
use std::thread;

/// Команда от UI к data layer.
#[derive(Debug)]
pub enum EditorCommand {
    /// Текст изменился (для дебаунса/throttle).
    TextChanged(String),
    /// Отформатировать текущий текст с заданными настройками.
    Format {
        content: String,
        renumber_step: u32,
        skip_empty_lines: bool,
    },
    /// Проверить текущий текст.
    Validate(String),
    /// Открыть файл.
    OpenFile,
    /// Сохранить файл.
    SaveFile {
        path: Option<String>,
        content: String,
    },
    /// Результат выбора файла из диалога.
    FilePickerResult {
        path: Option<String>,
        mode: FilePickerMode,
    },
    /// Результат диалога подтверждения.
    DialogResult { confirmed: bool },
}

/// Событие от data layer к UI.
#[derive(Debug)]
pub enum EditorEvent {
    /// Результат форматирования.
    Formatted {
        content: String,
        errors: Vec<crate::shared::ValidationMessage>,
        /// Путь к файлу (если был открыт через FilePickerResult)
        file_path: Option<String>,
    },
    /// Результат валидации.
    Validated {
        errors: Vec<crate::shared::ValidationMessage>,
    },
    /// Запрос на открытие диалога выбора файла.
    RequestFilePicker { mode: FilePickerMode },
    /// Запрос на подтверждение.
    RequestDialog { title: String, message: String },
    /// Уведомление пользователя.
    NotifyUser { message: String, level: NotifyLevel },
    /// Data layer завершил обработку (idle).
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilePickerMode {
    Open,
    Save,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotifyLevel {
    Info,
    Warning,
    Error,
}

/// Запускает data layer в отдельном потоке.
/// Возвращает (tx для отправки команд, rx для получения событий).
pub fn spawn_data_layer() -> (mpsc::Sender<EditorCommand>, mpsc::Receiver<EditorEvent>) {
    let (cmd_tx, cmd_rx) = mpsc::channel::<EditorCommand>();
    let (evt_tx, evt_rx) = mpsc::channel::<EditorEvent>();

    thread::spawn(move || {
        let mut data = DataLayer::new(evt_tx);

        loop {
            // Skip-pending: drain-им канал, оставляем только последнее сообщение TextChanged
            let mut cmd: Option<EditorCommand> = None;
            while let Ok(c) = cmd_rx.try_recv() {
                // coalesce TextChanged: если пришло несколько TextChanged подряд,
                // оставляем только последний
                match (&cmd, &c) {
                    (Some(EditorCommand::TextChanged(_)), EditorCommand::TextChanged(_)) => {
                        cmd = Some(c);
                    }
                    _ => {
                        // Если уже есть команда не TextChanged, сначала обрабатываем её
                        if cmd.is_some() {
                            data.process(&cmd.take().unwrap());
                        }
                        cmd = Some(c);
                    }
                }
            }

            // Обрабатываем последнюю команду
            if let Some(cmd) = cmd {
                data.process(&cmd);
            } else {
                // Нет команд — небольшая пауза, чтобы не грузить CPU
                thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    });

    (cmd_tx, evt_rx)
}

/// Внутреннее состояние data layer.
struct DataLayer {
    evt_tx: mpsc::Sender<EditorEvent>,
    pending_file_picker: bool,
    pending_dialog: bool,
    /// Текущий путь к файлу (для сохранения)
    current_file_path: Option<String>,
    /// Содержимое, ожидающее сохранения (для SaveAs)
    pending_save_content: Option<String>,
}

impl DataLayer {
    fn new(evt_tx: mpsc::Sender<EditorEvent>) -> Self {
        Self {
            evt_tx,
            pending_file_picker: false,
            pending_dialog: false,
            current_file_path: None,
            pending_save_content: None,
        }
    }

    fn process(&mut self, cmd: &EditorCommand) {
        match cmd {
            EditorCommand::TextChanged(_content) => {
                // Пока ничего не делаем — форматируем только по явной команде
            }
            EditorCommand::Format {
                content,
                renumber_step,
                skip_empty_lines,
            } => {
                self.send(EditorEvent::NotifyUser {
                    message: "Форматирование...".to_string(),
                    level: NotifyLevel::Info,
                });
                let result = pipeline::format_code(content, *renumber_step, *skip_empty_lines);
                match result {
                    Ok((formatted, errors)) => {
                        self.send(EditorEvent::Formatted {
                            content: formatted,
                            errors,
                            file_path: None,
                        });
                    }
                    Err(e) => {
                        self.send(EditorEvent::NotifyUser {
                            message: format!("Ошибка: {}", e),
                            level: NotifyLevel::Error,
                        });
                    }
                }
                self.send(EditorEvent::Idle);
            }
            EditorCommand::Validate(content) => {
                self.send(EditorEvent::NotifyUser {
                    message: "Проверка...".to_string(),
                    level: NotifyLevel::Info,
                });
                let result = pipeline::validate_code(content);
                match result {
                    Ok(errors) => {
                        self.send(EditorEvent::Validated { errors });
                    }
                    Err(e) => {
                        self.send(EditorEvent::NotifyUser {
                            message: format!("Ошибка: {}", e),
                            level: NotifyLevel::Error,
                        });
                    }
                }
                self.send(EditorEvent::Idle);
            }
            EditorCommand::OpenFile => {
                self.send(EditorEvent::RequestFilePicker {
                    mode: FilePickerMode::Open,
                });
                self.pending_file_picker = true;
            }
            EditorCommand::SaveFile { path, content } => {
                let file_path = match path {
                    Some(p) => p.clone(),
                    None => {
                        self.pending_save_content = Some(content.clone());
                        self.send(EditorEvent::RequestFilePicker {
                            mode: FilePickerMode::Save,
                        });
                        self.pending_file_picker = true;
                        return;
                    }
                };
                self.current_file_path = Some(file_path.clone());
                match std::fs::write(&file_path, content) {
                    Ok(_) => {
                        self.send(EditorEvent::NotifyUser {
                            message: format!("Сохранено: {}", file_path),
                            level: NotifyLevel::Info,
                        });
                        // Отправляем Formatted с file_path, чтобы UI обновил путь
                        self.send(EditorEvent::Formatted {
                            content: String::new(),
                            errors: vec![],
                            file_path: Some(file_path.clone()),
                        });
                    }
                    Err(e) => {
                        self.send(EditorEvent::NotifyUser {
                            message: format!("Ошибка сохранения: {}", e),
                            level: NotifyLevel::Error,
                        });
                    }
                }
                self.send(EditorEvent::Idle);
            }
            EditorCommand::FilePickerResult { path, mode } => {
                self.pending_file_picker = false;
                match mode {
                    FilePickerMode::Open => {
                        if let Some(path) = path {
                            self.current_file_path = Some(path.clone());
                            match std::fs::read_to_string(&path) {
                                Ok(content) => {
                                    let content = content
                                        .replace("\r\n", "\n")
                                        .replace("\r", "\n")
                                        .trim_start_matches('\u{feff}')
                                        .to_string();
                                    self.send(EditorEvent::Formatted {
                                        content,
                                        errors: vec![],
                                        file_path: Some(path.clone()),
                                    });
                                }
                                Err(e) => {
                                    self.send(EditorEvent::NotifyUser {
                                        message: format!("Ошибка: {}", e),
                                        level: NotifyLevel::Error,
                                    });
                                }
                            }
                        }
                        self.send(EditorEvent::Idle);
                    }
                    FilePickerMode::Save => {
                        if let Some(path) = path {
                            self.current_file_path = Some(path.clone());
                            let content = self.pending_save_content.take().unwrap_or_default();
                            match std::fs::write(&path, &content) {
                                Ok(_) => {
                                    self.send(EditorEvent::NotifyUser {
                                        message: format!("Сохранено: {}", path),
                                        level: NotifyLevel::Info,
                                    });
                                    self.send(EditorEvent::Formatted {
                                        content: String::new(),
                                        errors: vec![],
                                        file_path: Some(path.clone()),
                                    });
                                }
                                Err(e) => {
                                    self.send(EditorEvent::NotifyUser {
                                        message: format!("Ошибка сохранения: {}", e),
                                        level: NotifyLevel::Error,
                                    });
                                }
                            }
                        }
                        self.pending_save_content = None;
                        self.send(EditorEvent::Idle);
                    }
                }
            }
            EditorCommand::DialogResult { confirmed: _ } => {
                self.pending_dialog = false;
                // Результат диалога обрабатывается в UI
            }
        }
    }

    fn send(&self, event: EditorEvent) {
        let _ = self.evt_tx.send(event);
    }
}

mod pipeline;
