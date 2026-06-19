//! Update — редьюсер: применяет намерения к модели

use crate::interfaces::gui::intent::AxisSwapMode;
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
            Intent::ToggleAxisSwap => {
                self.set_axis_swap_open(!self.axis_swap_open());
            }
            Intent::SetAxisSwapMode(mode) => {
                self.set_axis_swap_mode(mode.clone());
            }
            Intent::SetSwapAxis1(axis) => {
                self.set_axis_swap_axis1(axis.clone());
            }
            Intent::SetSwapAxis2(axis) => {
                self.set_axis_swap_axis2(axis.clone());
            }
            Intent::SetInvertAxis(axis) => {
                self.set_axis_invert_axis(axis.clone());
            }
            Intent::ApplyAxisSwap => {
                let content = self.content().to_string();
                let result = match self.axis_swap_mode() {
                    AxisSwapMode::Swap => {
                        swap_axes(&content, self.axis_swap_axis1(), self.axis_swap_axis2())
                    }
                    AxisSwapMode::Invert => {
                        invert_axes_by_letter(&content, self.axis_invert_axis())
                    }
                };
                self.set_content(result);
                self.set_modified(true);
                self.set_axis_swap_open(false);
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

/// Меняет местами две оси в тексте G-кода.
/// Например, swap Z и X: "G0 X10 Z20" → "G0 Z10 X20"
pub(crate) fn swap_axes(text: &str, axis1: &str, axis2: &str) -> String {
    if axis1.len() != 1 || axis2.len() != 1 || axis1 == axis2 {
        return text.to_string();
    }
    // Построчный проход, чтобы не задеть комментарии
    let mut result = String::with_capacity(text.len());
    for line in text.lines() {
        if result.is_empty() {
            result.push_str(&swap_in_line(line, axis1, axis2));
        } else {
            result.push('\n');
            result.push_str(&swap_in_line(line, axis1, axis2));
        }
    }
    // Сохраняем trailing newline
    if text.ends_with('\n') {
        result.push('\n');
    }
    result
}

/// Меняет местами оси в пределах одной строки (до символа комментария).
fn swap_in_line(line: &str, axis1: &str, axis2: &str) -> String {
    let (code_part, comment) = match line.split_once(';') {
        Some((c, rest)) => (c, Some(rest)),
        None => (line, None),
    };
    // Сначала отделяем содержимое в кавычках — оно не меняется
    let mut result = String::new();
    let mut remaining = code_part;
    while let Some(quote_start) = remaining.find('"') {
        result.push_str(&remaining[..quote_start]);
        remaining = &remaining[quote_start + 1..];
        if let Some(quote_end) = remaining.find('"') {
            // Возвращаем содержимое кавычек как есть
            let quoted = &remaining[..quote_end];
            // Применяем swap к тексту перед очередной кавычкой
            // а содержимое кавычек оставляем
            remaining = &remaining[quote_end + 1..];
            // Мы уже обработали текст перед кавычкой выше, теперь
            // добавляем кавычки и содержимое
            result.push('"');
            result.push_str(quoted);
            result.push('"');
        } else {
            // Нет закрывающей кавычки — остаток как есть
            result.push('"');
            result.push_str(remaining);
            remaining = "";
            break;
        }
    }
    // Обрабатываем остаток (вне кавычек)
    let words: Vec<&str> = remaining.split_whitespace().collect();
    let mut swapped: Vec<String> = Vec::new();
    for word in &words {
        let w = *word;
        if let Some((ax, expr)) = split_axis_expr(w) {
            if ax == axis1 {
                swapped.push(format!("{axis2}={expr}"));
            } else if ax == axis2 {
                swapped.push(format!("{axis1}={expr}"));
            } else {
                swapped.push(w.to_string());
            }
        } else {
            let (prefix, value) = split_axis_value(w);
            if prefix == axis1 {
                swapped.push(format!("{axis2}{value}"));
            } else if prefix == axis2 {
                swapped.push(format!("{axis1}{value}"));
            } else {
                swapped.push(w.to_string());
            }
        }
    }
    if !swapped.is_empty() {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&swapped.join(" "));
    }
    if let Some(com) = comment {
        result.push(';');
        result.push_str(com);
    }
    result
}

/// Выделяет букву оси и значение из токена вида "X10.5" или "Z-20".
fn split_axis_value(word: &str) -> (&str, &str) {
    let axis_letters = ['X', 'Y', 'Z', 'A', 'B', 'C', 'U', 'V', 'W'];
    let mut chars = word.char_indices();
    let Some((_, first_char)) = chars.next() else {
        return (word, "");
    };
    if !axis_letters.contains(&first_char)
        && !axis_letters.contains(&first_char.to_ascii_lowercase())
    {
        return (word, "");
    }
    // Второй символ — начало значения (переносим safe byte index)
    let value_start = chars.next().map(|(i, _)| i).unwrap_or(word.len());
    (&word[..value_start], &word[value_start..])
}

/// Выделяет букву оси и выражение из AxisExpr вида "Z=71.304" или "X=160+10".
fn split_axis_expr(word: &str) -> Option<(&str, &str)> {
    let axis_letters = ['X', 'Y', 'Z', 'A', 'B', 'C', 'U', 'V', 'W'];
    if word.len() < 3 || !word.contains('=') {
        return None;
    }
    let fc = word.chars().next().unwrap();
    if !axis_letters.contains(&fc) && !axis_letters.contains(&fc.to_ascii_lowercase()) {
        return None;
    }
    if word.as_bytes()[1] == b'=' {
        Some((&word[..1], &word[2..]))
    } else {
        None
    }
}

/// Инвертирует выражение: "1+10" → "-(1+10)", "-20-10" → "-(-20-10)", "71.304" → "-71.304".
fn invert_expression(expr: &str) -> String {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    // Проверяем операторы в теле (после возможного начального минуса)
    let check_body = if let Some(stripped) = trimmed.strip_prefix('-') {
        stripped
    } else {
        trimmed
    };
    let has_operator = check_body.contains('+')
        || check_body.contains('-')
        || check_body.contains('*')
        || check_body.contains('/');
    if has_operator {
        // Выражение с операторами — оборачиваем с минусом
        format!("-({})", trimmed)
    } else if let Some(stripped) = trimmed.strip_prefix('-') {
        // Простое отрицательное число/параметр: -71 → 71, -R20 → R20
        stripped.to_string()
    } else {
        // Простое положительное: 71 → -71, R20 → -R20
        format!("-{}", trimmed)
    }
}

/// Инвертирует знак у указанной оси в тексте G-кода.
pub(crate) fn invert_axes_by_letter(text: &str, axis_letter: &str) -> String {
    if axis_letter.len() != 1 {
        return text.to_string();
    }
    let target = axis_letter.chars().next().unwrap();
    let axis_letters = ['X', 'Y', 'Z', 'A', 'B', 'C', 'U', 'V', 'W'];
    if !axis_letters.contains(&target) && !axis_letters.contains(&target.to_ascii_lowercase()) {
        return text.to_string();
    }
    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut i = 0;
    let mut in_comment = false;
    let mut in_quotes = false;
    while i < bytes.len() {
        let ch = bytes[i] as char;
        if ch == '"' {
            in_quotes = !in_quotes;
            result.push(ch);
            i += 1;
            continue;
        }
        if in_quotes {
            result.push(ch);
            i += 1;
            continue;
        }
        if ch == ';' {
            in_comment = true;
            result.push(ch);
            i += 1;
            continue;
        }
        if ch == '\n' {
            in_comment = false;
            result.push(ch);
            i += 1;
            continue;
        }
        if in_comment {
            result.push(ch);
            i += 1;
            continue;
        }
        let upper = ch.to_ascii_uppercase();
        let target_upper = target.to_ascii_uppercase();
        if upper == target_upper
            && i + 1 < bytes.len()
            && (bytes[i + 1].is_ascii_digit()
                || bytes[i + 1] as char == '-'
                || bytes[i + 1] as char == '+')
        {
            let axis = ch;
            i += 1;
            let start = i;
            let negative = bytes[i] as char == '-';
            if negative || bytes[i] as char == '+' {
                i += 1;
            }
            while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] as char == '.') {
                i += 1;
            }
            if i > start {
                let value_str = &text[start..i];
                let inverted = if negative {
                    value_str[1..].to_string()
                } else {
                    format!("-{value_str}")
                };
                result.push(axis);
                result.push_str(&inverted);
            } else {
                result.push(axis);
                result.push_str(&text[start..i]);
            }
        } else if upper == target_upper && i + 2 < bytes.len() && bytes[i + 1] as char == '=' {
            let axis = ch;
            i += 2; // пропускаем '=', теперь i указывает после '='
            let start = i;
            if i < bytes.len() {
                // Читаем всё выражение до пробела/конца строки
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] as char != ';'
                {
                    i += 1;
                }
                if i > start {
                    let expr = &text[start..i];
                    let inverted = invert_expression(expr);
                    result.push(axis);
                    result.push('=');
                    result.push_str(&inverted);
                } else {
                    result.push(axis);
                    result.push('=');
                }
            } else {
                result.push(axis);
                result.push('=');
            }
        } else {
            result.push(ch);
            i += 1;
        }
    }
    result
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
