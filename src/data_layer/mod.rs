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
    Pipeline(PipelineCommand),
    File(FileCommand),
    Dialog(DialogCommand),
}

/// Команды пайплайна форматирования
#[derive(Debug)]
pub enum PipelineCommand {
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
}

/// Команды работы с файлами
#[derive(Debug)]
pub enum FileCommand {
    /// Открыть файл.
    OpenFile,
    /// Сохранить файл.
    SaveFile {
        path: Option<String>,
        content: String,
    },
}

/// Команды диалогов UI
#[derive(Debug)]
pub enum DialogCommand {
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
    Pipeline(PipelineEvent),
    File(FileEvent),
    Dialog(DialogEvent),
}

/// События пайплайна форматирования
#[derive(Debug)]
pub enum PipelineEvent {
    /// Результат форматирования.
    Formatted {
        content: String,
        errors: Vec<crate::shared::ValidationMessage>,
    },
    /// Результат валидации.
    Validated {
        errors: Vec<crate::shared::ValidationMessage>,
    },
}

/// События работы с файлами
#[derive(Debug)]
pub enum FileEvent {
    /// Файл загружен.
    Loaded { content: String, file_path: String },
    /// Файл сохранён.
    Saved { file_path: String },
}

/// События диалогов и уведомлений
#[derive(Debug)]
pub enum DialogEvent {
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
                    (
                        Some(EditorCommand::Pipeline(PipelineCommand::TextChanged(_))),
                        EditorCommand::Pipeline(PipelineCommand::TextChanged(_)),
                    ) => {
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
            EditorCommand::Pipeline(p) => self.handle_pipeline(p),
            EditorCommand::File(f) => self.handle_file(f),
            EditorCommand::Dialog(d) => self.handle_dialog(d),
        }
    }

    fn handle_pipeline(&mut self, cmd: &PipelineCommand) {
        match cmd {
            PipelineCommand::TextChanged(_content) => {
                // Пока ничего не делаем — форматируем только по явной команде
            }
            PipelineCommand::Format {
                content,
                renumber_step,
                skip_empty_lines,
            } => {
                self.send(EditorEvent::Dialog(DialogEvent::NotifyUser {
                    message: "Форматирование...".to_string(),
                    level: NotifyLevel::Info,
                }));
                let result = pipeline::format_code(content, *renumber_step, *skip_empty_lines);
                match result {
                    Ok((formatted, errors)) => {
                        self.send(EditorEvent::Pipeline(PipelineEvent::Formatted {
                            content: formatted,
                            errors,
                        }));
                    }
                    Err(e) => {
                        self.send(EditorEvent::Dialog(DialogEvent::NotifyUser {
                            message: format!("Ошибка: {}", e),
                            level: NotifyLevel::Error,
                        }));
                    }
                }
                self.send(EditorEvent::Dialog(DialogEvent::Idle));
            }
            PipelineCommand::Validate(content) => {
                self.send(EditorEvent::Dialog(DialogEvent::NotifyUser {
                    message: "Проверка...".to_string(),
                    level: NotifyLevel::Info,
                }));
                let result = pipeline::validate_code(content);
                match result {
                    Ok(errors) => {
                        self.send(EditorEvent::Pipeline(PipelineEvent::Validated { errors }));
                    }
                    Err(e) => {
                        self.send(EditorEvent::Dialog(DialogEvent::NotifyUser {
                            message: format!("Ошибка: {}", e),
                            level: NotifyLevel::Error,
                        }));
                    }
                }
                self.send(EditorEvent::Dialog(DialogEvent::Idle));
            }
        }
    }

    fn handle_file(&mut self, cmd: &FileCommand) {
        match cmd {
            FileCommand::OpenFile => {
                self.send(EditorEvent::Dialog(DialogEvent::RequestFilePicker {
                    mode: FilePickerMode::Open,
                }));
                self.pending_file_picker = true;
            }
            FileCommand::SaveFile { path, content } => {
                let file_path = match path {
                    Some(p) => p.clone(),
                    None => {
                        self.pending_save_content = Some(content.clone());
                        self.send(EditorEvent::Dialog(DialogEvent::RequestFilePicker {
                            mode: FilePickerMode::Save,
                        }));
                        self.pending_file_picker = true;
                        return;
                    }
                };
                self.current_file_path = Some(file_path.clone());
                match std::fs::write(&file_path, content) {
                    Ok(_) => {
                        self.send(EditorEvent::File(FileEvent::Saved {
                            file_path: file_path.clone(),
                        }));
                        self.send(EditorEvent::Dialog(DialogEvent::NotifyUser {
                            message: format!("Сохранено: {}", file_path),
                            level: NotifyLevel::Info,
                        }));
                    }
                    Err(e) => {
                        self.send(EditorEvent::Dialog(DialogEvent::NotifyUser {
                            message: format!("Ошибка сохранения: {}", e),
                            level: NotifyLevel::Error,
                        }));
                    }
                }
                self.send(EditorEvent::Dialog(DialogEvent::Idle));
            }
        }
    }

    fn handle_dialog(&mut self, cmd: &DialogCommand) {
        match cmd {
            DialogCommand::FilePickerResult { path, mode } => {
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
                                    self.send(EditorEvent::File(FileEvent::Loaded {
                                        content,
                                        file_path: path.clone(),
                                    }));
                                }
                                Err(e) => {
                                    self.send(EditorEvent::Dialog(DialogEvent::NotifyUser {
                                        message: format!("Ошибка: {}", e),
                                        level: NotifyLevel::Error,
                                    }));
                                }
                            }
                        }
                        self.send(EditorEvent::Dialog(DialogEvent::Idle));
                    }
                    FilePickerMode::Save => {
                        if let Some(path) = path {
                            self.current_file_path = Some(path.clone());
                            let content = self.pending_save_content.take().unwrap_or_default();
                            match std::fs::write(&path, &content) {
                                Ok(_) => {
                                    self.send(EditorEvent::File(FileEvent::Saved {
                                        file_path: path.clone(),
                                    }));
                                    self.send(EditorEvent::Dialog(DialogEvent::NotifyUser {
                                        message: format!("Сохранено: {}", path),
                                        level: NotifyLevel::Info,
                                    }));
                                }
                                Err(e) => {
                                    self.send(EditorEvent::Dialog(DialogEvent::NotifyUser {
                                        message: format!("Ошибка сохранения: {}", e),
                                        level: NotifyLevel::Error,
                                    }));
                                }
                            }
                        }
                        self.pending_save_content = None;
                        self.send(EditorEvent::Dialog(DialogEvent::Idle));
                    }
                }
            }
            DialogCommand::DialogResult { confirmed: _ } => {
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
