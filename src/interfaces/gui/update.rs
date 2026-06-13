//! Update — редьюсер: применяет намерения к модели

use super::intent::Intent;
use super::model::Model;

impl Model {
    /// Применяет намерение пользователя к модели.
    /// Вызывается из App::update() для каждого Intent.
    pub fn apply(&mut self, intent: &Intent) {
        match intent {
            Intent::CloseFile => {
                if self.modified && !self.file_path.is_empty() {
                    self.show_exit_dialog = true;
                    self.pending_action = Some(super::model::PendingAction::CloseFile);
                } else {
                    self.content.clear();
                    self.file_path.clear();
                    self.modified = false;
                    self.status = "Файл закрыт.".to_string();
                }
            }
            Intent::Exit => {
                if self.modified && !self.file_path.is_empty() {
                    self.show_exit_dialog = true;
                    self.pending_action = Some(super::model::PendingAction::Exit);
                } else {
                    std::process::exit(0);
                }
            }
            Intent::Format => {}
            Intent::Validate => {}
            Intent::OpenFile => {}
            Intent::SaveFile => {}
            Intent::SaveAs => {}
            Intent::ToggleSettings => {
                self.settings_open = !self.settings_open;
            }
            Intent::SetRenumberStep(step) => {
                self.format_settings.renumber_step = *step;
                self.save_settings();
            }
            Intent::SetSkipEmptyLines(skip) => {
                self.format_settings.skip_empty_lines = *skip;
                self.save_settings();
            }
            Intent::ConfirmSave => {}
            Intent::DiscardAndContinue => {}
            Intent::CancelAction => {
                self.show_exit_dialog = false;
                self.pending_action = None;
            }
        }
    }
}

// --- Сохранение и загрузка настроек ---

fn settings_path() -> std::path::PathBuf {
    let mut path = if let Some(home) = std::env::var("HOME").ok() {
        std::path::PathBuf::from(home)
    } else {
        std::path::PathBuf::from(".")
    };
    path.push(".config");
    path.push("gcode-editor");
    path.push("settings.json");
    path
}

impl Model {
    /// Сохраняет настройки в файл
    pub fn save_settings(&self) {
        let path = settings_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.format_settings) {
            let _ = std::fs::write(&path, json);
        }
    }

    /// Загружает настройки из файла
    pub fn load_settings(&mut self) {
        let path = settings_path();
        if let Ok(json) = std::fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str::<super::model::FormatSettings>(&json) {
                self.format_settings = settings;
            }
        }
    }
}
