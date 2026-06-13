//! Update — редьюсер: применяет намерения к модели

use super::intent::Intent;
use super::model::Model;
use crate::shared::i18n;

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
            Intent::SetLanguage(lang) => {
                self.format_settings.language = lang.clone();
                i18n::set_lang(lang);
                self.save_settings();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interfaces::gui::model::PendingAction;

    fn make_model() -> Model {
        Model {
            content: "G0 X10 Y20".to_string(),
            file_path: "/path/to/file.nc".to_string(),
            modified: true,
            ..Default::default()
        }
    }

    // -----------------------------------------------------------------------
    // CloseFile
    // -----------------------------------------------------------------------

    #[test]
    fn test_close_file_modified_shows_dialog() {
        let mut m = make_model();
        m.apply(&Intent::CloseFile);
        assert!(m.show_exit_dialog);
        assert_eq!(m.pending_action, Some(PendingAction::CloseFile));
        assert_eq!(m.content, "G0 X10 Y20"); // не очищается до подтверждения
    }

    #[test]
    fn test_close_file_not_modified_clears() {
        let mut m = Model {
            content: "G0 X10".to_string(),
            file_path: "/path/file.nc".to_string(),
            modified: false,
            ..Default::default()
        };
        m.apply(&Intent::CloseFile);
        assert!(!m.show_exit_dialog);
        assert!(m.content.is_empty());
        assert!(m.file_path.is_empty());
        assert!(!m.modified);
    }

    #[test]
    fn test_close_file_no_path_clears() {
        let mut m = Model {
            content: "G0 X10".to_string(),
            modified: true,
            ..Default::default()
        };
        // Если file_path пуст — сразу очищаем, без диалога
        assert!(m.file_path.is_empty());
        m.apply(&Intent::CloseFile);
        assert!(!m.show_exit_dialog);
        assert!(m.content.is_empty());
    }

    // -----------------------------------------------------------------------
    // Exit
    // -----------------------------------------------------------------------

    #[test]
    fn test_exit_modified_shows_dialog() {
        let mut m = make_model();
        m.apply(&Intent::Exit);
        assert!(m.show_exit_dialog);
        assert_eq!(m.pending_action, Some(PendingAction::Exit));
    }

    // Exit без modified вызывает std::process::exit — не тестируем

    // -----------------------------------------------------------------------
    // ToggleSettings
    // -----------------------------------------------------------------------

    #[test]
    fn test_toggle_settings_twice() {
        let mut m = Model::default();
        assert!(!m.settings_open);
        m.apply(&Intent::ToggleSettings);
        assert!(m.settings_open);
        m.apply(&Intent::ToggleSettings);
        assert!(!m.settings_open);
    }

    // -----------------------------------------------------------------------
    // SetRenumberStep
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_renumber_step() {
        let mut m = Model::default();
        assert_eq!(m.format_settings.renumber_step, 1);
        m.apply(&Intent::SetRenumberStep(10));
        assert_eq!(m.format_settings.renumber_step, 10);
        m.apply(&Intent::SetRenumberStep(100));
        assert_eq!(m.format_settings.renumber_step, 100);
    }

    // -----------------------------------------------------------------------
    // SetSkipEmptyLines
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_skip_empty_lines() {
        let mut m = Model::default();
        assert!(m.format_settings.skip_empty_lines);
        m.apply(&Intent::SetSkipEmptyLines(false));
        assert!(!m.format_settings.skip_empty_lines);
        m.apply(&Intent::SetSkipEmptyLines(true));
        assert!(m.format_settings.skip_empty_lines);
    }

    // -----------------------------------------------------------------------
    // CancelAction
    // -----------------------------------------------------------------------

    #[test]
    fn test_cancel_action() {
        let mut m = make_model();
        m.apply(&Intent::CloseFile); // открыли диалог
        assert!(m.show_exit_dialog);
        assert_eq!(m.pending_action, Some(PendingAction::CloseFile));

        m.apply(&Intent::CancelAction);
        assert!(!m.show_exit_dialog);
        assert_eq!(m.pending_action, None);
    }

    // -----------------------------------------------------------------------
    // SetLanguage
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_language_en() {
        let mut m = Model::default();
        assert_eq!(m.format_settings.language, "ru");
        m.apply(&Intent::SetLanguage("en".to_string()));
        assert_eq!(m.format_settings.language, "en");
        assert_eq!(i18n::current_lang(), "en");
        // Сброс, чтобы не влиять на другие тесты
        i18n::set_lang("ru");
    }

    #[test]
    fn test_set_language_ru() {
        let mut m = Model::default();
        m.apply(&Intent::SetLanguage("en".to_string()));
        m.apply(&Intent::SetLanguage("ru".to_string()));
        assert_eq!(m.format_settings.language, "ru");
        assert_eq!(i18n::current_lang(), "ru");
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
        let original = m.status.clone();
        m.apply(&Intent::Format);
        m.apply(&Intent::Validate);
        m.apply(&Intent::OpenFile);
        m.apply(&Intent::SaveFile);
        m.apply(&Intent::SaveAs);
        m.apply(&Intent::ConfirmSave);
        m.apply(&Intent::DiscardAndContinue);
        assert_eq!(m.status, original);
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
