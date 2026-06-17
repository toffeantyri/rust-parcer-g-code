//! Update — редьюсер: применяет намерения к модели

use crate::interfaces::gui::intent::Intent;
use crate::interfaces::gui::model::{Model, PendingAction};
use crate::shared::i18n;

impl Model {
    /// Применяет намерение пользователя к модели.
    /// Вызывается из App::update() для каждого Intent.
    pub fn apply(&mut self, intent: &Intent) {
        match intent {
            Intent::CloseFile => {
                if self.modified() && !self.file_path().is_empty() {
                    self.set_show_exit_dialog(true);
                    self.set_pending_action(Some(PendingAction::CloseFile));
                } else {
                    self.set_content(String::new());
                    self.set_file_path(String::new());
                    self.set_modified(false);
                    self.set_status(i18n::locale().status.file_closed.clone());
                }
            }
            Intent::Exit => {
                if self.modified() && !self.file_path().is_empty() {
                    self.set_show_exit_dialog(true);
                    self.set_pending_action(Some(PendingAction::Exit));
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
                self.set_settings_open(!self.settings_open());
            }
            Intent::SetRenumberStep(step) => {
                let mut settings = self.format_settings().clone();
                settings.renumber_step = *step;
                self.set_format_settings(settings);
                self.save_settings();
            }
            Intent::SetSkipEmptyLines(skip) => {
                let mut settings = self.format_settings().clone();
                settings.skip_empty_lines = *skip;
                self.set_format_settings(settings);
                self.save_settings();
            }
            Intent::ConfirmSave => {}
            Intent::DiscardAndContinue => {}
            Intent::CancelAction => {
                self.set_show_exit_dialog(false);
                self.set_pending_action(None);
            }
            Intent::ToggleShortcuts => {
                self.set_shortcuts_open(!self.shortcuts_open());
            }
            Intent::SetLanguage(lang) => {
                let mut settings = self.format_settings().clone();
                settings.language = lang.clone();
                self.set_format_settings(settings);
                i18n::set_lang(lang);
                self.save_settings();
            }
        }
    }
}

/// Функция сохранения настроек
impl Model {
    pub fn save_settings(&self) {
        let path = settings_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.format_settings()) {
            let _ = std::fs::write(&path, json);
        }
    }

    pub fn load_settings(&mut self) {
        let path = settings_path();
        if let Ok(json) = std::fs::read_to_string(&path) {
            if let Ok(settings) =
                serde_json::from_str::<crate::interfaces::gui::model::FormatSettings>(&json)
            {
                self.set_format_settings(settings);
            }
        }
    }
}

#[cfg(test)]
#[path = "update_tests.rs"]
mod tests;

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
