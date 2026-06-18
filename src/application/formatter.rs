//! Прикладной слой: форматтер — преобразует AST в отформатированный текст G-кода

use crate::domain::{IfStatement, Statement, WhileStatement};

pub struct FormatConfig {
    pub indent_size: usize,
    pub use_spaces: bool,
    pub uppercase_codes: bool,
    pub decimal_places: usize,
    pub renumber_step: u32,
    pub skip_empty_lines: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        FormatConfig {
            indent_size: 2,
            use_spaces: true,
            uppercase_codes: true,
            decimal_places: 5,
            renumber_step: 0,
            skip_empty_lines: true,
        }
    }
}

pub struct Formatter {
    config: FormatConfig,
}

/// Отбрасывает незначащие нули после десятичной точки.
/// Примеры: "X10.000" → "X10", "X10.100" → "X10.1", "X-5.500" → "X-5.5"
fn trim_trailing_zeros(input: String) -> String {
    if !input.contains('.') {
        return input;
    }
    let dot_pos = input.find('.').unwrap();
    let prefix = &input[..dot_pos];
    let fractional = &input[dot_pos + 1..];
    let trimmed = fractional.trim_end_matches('0');
    if trimmed.is_empty() {
        prefix.to_string()
    } else {
        format!("{}.{}", prefix, trimmed)
    }
}

impl Formatter {
    pub fn new(config: FormatConfig) -> Self {
        Formatter { config }
    }

    pub fn format_program(&self, program: &[Statement]) -> String {
        let formatted = self.format_block(program, 0);
        if self.config.renumber_step > 0 {
            let formatted = apply_renumbering(
                &formatted,
                self.config.renumber_step,
                self.config.skip_empty_lines,
            );
            if self.config.skip_empty_lines {
                normalize_blank_lines(&formatted)
            } else {
                formatted
            }
        } else {
            formatted
        }
    }

    fn format_block(&self, program: &[Statement], indent_level: usize) -> String {
        let mut result = String::new();
        let mut line_parts: Vec<String> = Vec::new();

        for stmt in program {
            match stmt {
                Statement::WhileBlock(w) => {
                    if !line_parts.is_empty() {
                        result.push_str(&self.format_line(&line_parts));
                        line_parts.clear();
                    }
                    result.push_str(&self.format_while(w, indent_level));
                }
                Statement::IfBlock(i) => {
                    if !line_parts.is_empty() {
                        result.push_str(&self.format_line(&line_parts));
                        line_parts.clear();
                    }
                    result.push_str(&self.format_if(i, indent_level));
                }
                Statement::NewLine => {
                    if !line_parts.is_empty() {
                        result.push_str(&self.format_line(&line_parts));
                        line_parts.clear();
                    } else {
                        result.push('\n');
                    }
                }
                Statement::NCode(_code) => {
                    if !line_parts.is_empty() {
                        result.push_str(&self.format_line(&line_parts));
                        line_parts.clear();
                    }
                    line_parts.push(self.format_statement(stmt));
                }
                Statement::Raw(raw) => {
                    if !line_parts.is_empty() {
                        result.push_str(&self.format_line(&line_parts));
                        line_parts.clear();
                    }
                    result.push_str(raw);
                    result.push('\n');
                }
                _ => {
                    line_parts.push(self.format_statement(stmt));
                }
            }
        }

        if !line_parts.is_empty() {
            result.push_str(&self.format_line(&line_parts));
        }

        result
    }

    fn format_while(&self, w: &WhileStatement, indent_level: usize) -> String {
        let indent = self.make_indent(indent_level);
        let body_indent = self.make_indent(indent_level + 1);
        let mut out = String::new();

        // WHILE без N-кода
        out.push_str(&format!("{}WHILE {}\n", indent, w.condition));

        // Тело
        let body = self.format_block(&w.body, indent_level + 1);
        let body = body.trim_start_matches('\n');
        let body = body.trim_end_matches('\n');
        if !body.is_empty() {
            for line in body.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    out.push('\n');
                } else {
                    out.push_str(&format!("{}{}\n", body_indent, line));
                }
            }
        }

        // ENDWHILE — на уровне тела
        out.push_str(&format!("{}ENDWHILE\n", body_indent));

        out
    }

    fn format_if(&self, i: &IfStatement, indent_level: usize) -> String {
        let indent = self.make_indent(indent_level);
        let body_indent = self.make_indent(indent_level + 1);
        let mut out = String::new();

        // IF без N-кода
        out.push_str(&format!("{}IF {}\n", indent, i.condition));

        // THEN
        let then_body = self.format_block(&i.then_body, indent_level + 1);
        let then_lines: Vec<&str> = then_body.lines().collect();
        for line in &then_lines {
            if line.trim().is_empty() {
                out.push('\n');
            } else {
                out.push_str(&format!("{}{}\n", body_indent, line));
            }
        }

        // ELSE
        if let Some(ref else_body) = i.else_body {
            out.push_str(&format!("{}ELSE\n", indent));
            let else_fmt = self.format_block(else_body, indent_level + 1);
            let else_lines: Vec<&str> = else_fmt.lines().collect();
            for line in &else_lines {
                if line.trim().is_empty() {
                    out.push('\n');
                } else {
                    out.push_str(&format!("{}{}\n", body_indent, line));
                }
            }
        }

        // ENDIF — на уровне тела
        out.push_str(&format!("{}ENDIF\n", body_indent));

        out
    }

    fn make_indent(&self, level: usize) -> String {
        let size = self.config.indent_size * level;
        if self.config.use_spaces {
            " ".repeat(size)
        } else {
            "\t".repeat(level)
        }
    }

    fn format_statement(&self, stmt: &Statement) -> String {
        match stmt {
            Statement::Motion(m) => {
                let prefix = if self.config.uppercase_codes {
                    "G"
                } else {
                    "g"
                };
                format!("{}{}", prefix, m.code)
            }
            Statement::NCode(code) => format!("N{:04}", code),
            Statement::Speed(s) => s.clone(),
            Statement::RParameter(r) => r.clone(),
            Statement::Word(word) => word.clone(),
            Statement::Misc(m) => {
                let prefix = if self.config.uppercase_codes {
                    "M"
                } else {
                    "m"
                };
                format!("{}{}", prefix, m.code)
            }
            Statement::Axis(a) => {
                if let Some(v) = a.value {
                    if let Some(prec) = a.decimal_places {
                        // Форматируем с заданным количеством знаков
                        let formatted = format!("{}{:.prec$}", a.axis, v, prec = prec);
                        // Отбрасываем незначащие нули и точку, если число целое
                        trim_trailing_zeros(formatted)
                    } else {
                        // Целое число — выводим без десятичной точки
                        format!("{}{}", a.axis, v)
                    }
                } else {
                    a.axis.clone()
                }
            }
            Statement::Comment(c) => format!(";{}", c.text),
            Statement::Raw(raw) => raw.clone(),
            Statement::WhileBlock(w) => format!("WHILE {}", w.condition),
            Statement::IfBlock(i) => format!("IF {}", i.condition),
            Statement::NewLine => String::new(),
        }
    }

    fn format_line(&self, parts: &[String]) -> String {
        format!("{}\n", parts.join(" "))
    }
}

/// Удаляет N-код из начала строки, если он есть
fn trim_ncode(s: &str) -> &str {
    let s = s.trim_start();
    if s.starts_with('N') || s.starts_with('n') {
        // Пропускаем букву N/n и цифры
        let after_n = &s[1..];
        let digits: usize = after_n.chars().take_while(|c| c.is_ascii_digit()).count();
        if digits > 0 {
            &s[1 + digits..]
        } else {
            s
        }
    } else {
        s
    }
}

fn apply_renumbering(text: &str, step: u32, skip_empty_lines: bool) -> String {
    let mut result = String::new();
    let mut current_n: u32 = 0;

    for line in text.lines() {
        let trimmed = line.trim();

        // Пустая строка — нумеруем если skip_empty_lines=false
        if trimmed.is_empty() {
            if !skip_empty_lines {
                current_n += step;
                result.push_str(&format!("N{}\n", current_n));
            } else {
                result.push('\n');
            }
            continue;
        }

        // Если строка содержит только комментарий (начинается с ;) — не нумеруем
        if trimmed.starts_with(';') {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        // Skip empty lines: если строка содержит только старый N-код и больше ничего — удаляем N-код
        if skip_empty_lines {
            let without_n = trim_ncode(trimmed);
            let after_n = without_n.trim();
            if after_n.is_empty() {
                // Была строка только с N-кодом — удаляем, но счётчик увеличиваем
                // Не добавляем \n, чтобы следующая строка могла встать на ту же строку
                continue;
            }
            // Если после удаления N-кода остался только комментарий — не нумеруем
            if after_n.starts_with(';') {
                result.push_str(without_n.trim_start());
                result.push('\n');
                continue;
            }
        }

        // Удаляем старый N-код из начала строки, если он есть
        let clean_line = trim_ncode(line);
        // Сохраняем начальные пробелы (отступы) для вложенных блоков
        let indent: String = line
            .chars()
            .take_while(|c| c.is_whitespace() && *c != '\n')
            .collect();

        current_n += step;
        // N-код в начале строки, за ним — отступы и содержимое
        let content = clean_line.trim_start();
        result.push_str(&format!("N{} {}{}\n", current_n, indent, content));
    }

    result
}

/// Схлопывает множественные пустые строки в одну.
/// Оставляет не больше одной пустой строки подряд.
fn normalize_blank_lines(text: &str) -> String {
    let mut result = String::new();
    let mut prev_was_blank = false;

    for line in text.lines() {
        let trimmed = line.trim();
        let is_blank = trimmed.is_empty()
            || (trimmed.starts_with('N') || trimmed.starts_with('n'))
                && trimmed[1..].chars().all(|c| c.is_ascii_digit());

        if is_blank {
            if !prev_was_blank {
                // Оставляем одну пустую строку
                prev_was_blank = true;
            }
            // Вторую и последующие — пропускаем
        } else {
            if prev_was_blank {
                result.push('\n');
                prev_was_blank = false;
            }
            result.push_str(line);
            result.push('\n');
        }
    }

    if prev_was_blank {
        result.push('\n');
    }

    result
}

#[cfg(test)]
#[path = "formatter_tests.rs"]
mod tests;
