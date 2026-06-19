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
            Intent::ToggleSearch => {
                self.set_search_open(!self.search_open());
                if self.search_open() {
                    self.set_search_focus_needed(true);
                } else {
                    // Закрытие — сбрасываем состояние
                    self.set_search_index(0);
                    self.set_search_matches(Vec::new());
                    self.set_search_query(String::new());
                    self.set_search_last_query(String::new());
                }
            }
            Intent::ToggleReplace => {
                self.set_replace_open(!self.replace_open());
                if self.replace_open() {
                    self.set_replace_focus_needed(true);
                } else {
                    self.set_replace_index(0);
                    self.set_replace_matches(Vec::new());
                    self.set_replace_find(String::new());
                    self.set_replace_with(String::new());
                    self.set_replace_last_find(String::new());
                }
            }
            Intent::DoSearch => {
                let query = self.search_query().to_lowercase();
                if query.is_empty() {
                    self.set_search_matches(Vec::new());
                    self.set_search_last_query(String::new());
                } else {
                    let content_lower = self.content().to_lowercase();
                    self.set_search_matches(find_all_occurrences(&content_lower, &query));
                    self.set_search_last_query(query.clone());
                    self.set_search_index(0);
                }
            }
            Intent::FindNext => {
                // Если запрос изменился или вхождений нет — пересчитываем
                let query = self.search_query().to_lowercase();
                if query != *self.search_last_query() || self.search_matches().is_empty() {
                    self.apply(&Intent::DoSearch);
                } else {
                    let idx = (self.search_index() + 1) % self.search_matches().len();
                    self.set_search_index(idx);
                }
            }
            Intent::CloseSearchDialog => {
                self.set_search_open(false);
                self.set_replace_open(false);
                self.set_search_index(0);
                self.set_search_matches(Vec::new());
                self.set_search_query(String::new());
                self.set_search_last_query(String::new());
                self.set_replace_index(0);
                self.set_replace_matches(Vec::new());
                self.set_replace_find(String::new());
                self.set_replace_with(String::new());
                self.set_replace_last_find(String::new());
            }
            Intent::SetSearchQuery(query) => {
                self.set_search_query(query.clone());
            }
            Intent::SetReplaceFind(find) => {
                self.set_replace_find(find.clone());
                // При изменении строки поиска — сбрасываем предыдущий результат
                self.set_replace_last_find(String::new());
                self.set_replace_matches(Vec::new());
                self.set_replace_index(0);
            }
            Intent::SetReplaceWith(replace) => {
                self.set_replace_with(replace.clone());
            }
            Intent::DoReplaceSearch => {
                let find = self.replace_find().to_lowercase();
                if find.is_empty() {
                    self.set_replace_matches(Vec::new());
                    self.set_replace_last_find(String::new());
                    self.set_replace_index(0);
                } else {
                    let content_lower = self.content().to_lowercase();
                    self.set_replace_matches(find_all_occurrences(&content_lower, &find));
                    self.set_replace_last_find(find);
                    self.set_replace_index(0);
                }
            }
            Intent::ReplaceFindNext => {
                // Первый клик — пересчитываем, последующие — переходим к следующему
                let find = self.replace_find().to_lowercase();
                if find != *self.replace_last_find() || self.replace_matches().is_empty() {
                    self.apply(&Intent::DoReplaceSearch);
                } else {
                    let idx = (self.replace_index() + 1) % self.replace_matches().len();
                    self.set_replace_index(idx);
                }
            }
            Intent::ReplaceOne => {
                let find = self.replace_find().to_lowercase();
                // Пересчитываем вхождения если нужно
                if self.replace_last_find() != &find {
                    let content_lower = self.content().to_lowercase();
                    self.set_replace_matches(find_all_occurrences(&content_lower, &find));
                    self.set_replace_last_find(find.clone());
                    self.set_replace_index(0);
                }
                if self.replace_matches().is_empty() {
                    return;
                }
                let idx = self.replace_index();
                let byte_pos = self.replace_matches()[idx];
                let find_len = self.replace_find().len();
                // Замена в тексте
                let mut content = self.content().to_string();
                content.replace_range(byte_pos..byte_pos + find_len, &self.replace_with());
                self.set_content(content);
                self.set_modified(true);
                // Пересчитываем вхождения после замены
                let find = self.replace_find().to_lowercase();
                let content_lower = self.content().to_lowercase();
                self.set_replace_matches(find_all_occurrences(&content_lower, &find));
                self.set_replace_last_find(find);
                // Продвигаем индекс или сбрасываем
                if self.replace_matches().is_empty() {
                    self.set_replace_index(0);
                    let loc = i18n::locale();
                    self.set_search_status(loc.replace.replaced.clone());
                } else {
                    let new_idx = if idx >= self.replace_matches().len() {
                        0
                    } else {
                        idx
                    };
                    self.set_replace_index(new_idx);
                }
            }
            Intent::ReplaceAll => {
                let find = self.replace_find();
                if find.is_empty() {
                    return;
                }
                let find_lower = find.to_lowercase();
                let mut content = self.content().to_string();
                // Замена всех вхождений (справа налево, чтобы смещения не нарушались)
                let content_lower = content.to_lowercase();
                let mut matches: Vec<usize> = find_all_occurrences(&content_lower, &find_lower);
                matches.sort_unstable_by(|a, b| b.cmp(a)); // от большего к меньшему
                for &pos in &matches {
                    let end = pos + find.len();
                    if end <= content.len() {
                        content.replace_range(pos..end, &self.replace_with());
                    }
                }
                self.set_content(content);
                self.set_modified(true);
                self.set_replace_matches(Vec::new());
                self.set_replace_index(0);
                self.set_replace_last_find(String::new());
                self.set_search_status(i18n::locale().replace.replaced.clone());
            }
        }
    }

    /// Устанавливает сообщение в статусбар.
    fn set_search_status(&mut self, msg: String) {
        self.set_status(msg);
    }
}

/// Находит все вхождения подстроки (case-insensitive) и возвращает byte-позиции начала.
fn find_all_occurrences(haystack: &str, needle: &str) -> Vec<usize> {
    if needle.is_empty() {
        return Vec::new();
    }
    haystack.match_indices(needle).map(|(pos, _)| pos).collect()
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
    let mut path = if let Ok(home) = std::env::var("HOME") {
        std::path::PathBuf::from(home)
    } else {
        std::path::PathBuf::from(".")
    };
    path.push(".config");
    path.push("gcode-editor");
    path.push("settings.json");
    path
}
