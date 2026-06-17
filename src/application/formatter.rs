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
        self.format_block(program, 0)
    }

    fn format_block(&self, program: &[Statement], indent_level: usize) -> String {
        let mut result = String::new();
        let mut line_parts: Vec<String> = Vec::new();
        let mut current_n: u32 = 0;
        let mut line_has_ncode = false;

        for stmt in program {
            match stmt {
                Statement::WhileBlock(w) => {
                    // Если в line_parts только N-код и включена перенумерация
                    // со skip_empty_lines — удаляем N-код, оставляем пустую строку
                    if !line_parts.is_empty() {
                        let has_only_ncode = line_has_ncode && line_parts.len() == 1;
                        if self.config.renumber_step > 0
                            && self.config.skip_empty_lines
                            && has_only_ncode
                        {
                            result.push('\n');
                        } else {
                            result.push_str(&self.format_line(&line_parts));
                        }
                        line_parts.clear();
                        line_has_ncode = false;
                    }
                    result.push_str(&self.format_while(w, indent_level));
                }
                Statement::IfBlock(i) => {
                    if !line_parts.is_empty() {
                        let has_only_ncode = line_has_ncode && line_parts.len() == 1;
                        if self.config.renumber_step > 0
                            && self.config.skip_empty_lines
                            && has_only_ncode
                        {
                            result.push('\n');
                        } else {
                            result.push_str(&self.format_line(&line_parts));
                        }
                        line_parts.clear();
                        line_has_ncode = false;
                    }
                    result.push_str(&self.format_if(i, indent_level));
                }
                Statement::NewLine => {
                    if !line_parts.is_empty() {
                        let has_only_ncode = line_has_ncode && line_parts.len() == 1;
                        if self.config.renumber_step > 0
                            && has_only_ncode
                            && self.config.skip_empty_lines
                        {
                            line_parts.clear();
                            result.push('\n');
                        } else {
                            result.push_str(&self.format_line(&line_parts));
                            line_parts.clear();
                        }
                        line_has_ncode = false;
                    } else {
                        if self.config.renumber_step > 0 && !self.config.skip_empty_lines {
                            current_n += self.config.renumber_step;
                            result.push_str(&format!("N{}\n", current_n));
                        } else {
                            result.push('\n');
                        }
                    }
                }
                Statement::NCode(_code) => {
                    if self.config.renumber_step > 0 {
                        current_n += self.config.renumber_step;
                        if !line_parts.is_empty() {
                            result.push_str(&self.format_line(&line_parts));
                            line_parts.clear();
                        }
                        line_parts.push(format!("N{}", current_n));
                        line_has_ncode = true;
                    } else {
                        if !line_parts.is_empty() {
                            result.push_str(&self.format_line(&line_parts));
                            line_parts.clear();
                        }
                        line_parts.push(self.format_statement(stmt));
                        line_has_ncode = true;
                    }
                }
                Statement::Raw(raw) => {
                    if !line_parts.is_empty() {
                        result.push_str(&self.format_line(&line_parts));
                        line_parts.clear();
                        line_has_ncode = false;
                    }
                    result.push_str(raw);
                    result.push('\n');
                }
                _ => {
                    if self.config.renumber_step > 0 && line_parts.is_empty() && !line_has_ncode {
                        current_n += self.config.renumber_step;
                        line_parts.push(format!("N{}", current_n));
                        line_has_ncode = true;
                    }
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
        let mut lines: Vec<&str> = body.lines().collect();
        // Убираем trailing N-коды перед ENDWHILE
        if self.config.skip_empty_lines {
            while let Some(last) = lines.last() {
                let trimmed = last.trim();
                if trimmed.is_empty() || Self::is_only_ncode(trimmed) {
                    lines.pop();
                } else {
                    break;
                }
            }
        }
        for line in &lines {
            if line.trim().is_empty() {
                out.push('\n');
            } else {
                out.push_str(&format!("{}{}\n", body_indent, line));
            }
        }

        // ENDWHILE
        out.push_str(&format!("{}ENDWHILE\n", indent));

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
        let mut lines: Vec<&str> = then_body.lines().collect();
        if self.config.skip_empty_lines {
            while let Some(last) = lines.last() {
                let trimmed = last.trim();
                if trimmed.is_empty() || Self::is_only_ncode(trimmed) {
                    lines.pop();
                } else {
                    break;
                }
            }
        }
        for line in &lines {
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
            let mut lines: Vec<&str> = else_fmt.lines().collect();
            if self.config.skip_empty_lines {
                while let Some(last) = lines.last() {
                    let trimmed = last.trim();
                    if trimmed.is_empty() || Self::is_only_ncode(trimmed) {
                        lines.pop();
                    } else {
                        break;
                    }
                }
            }
            for line in &lines {
                if line.trim().is_empty() {
                    out.push('\n');
                } else {
                    out.push_str(&format!("{}{}\n", body_indent, line));
                }
            }
        }

        // ENDIF
        out.push_str(&format!("{}ENDIF\n", indent));

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

    /// Проверяет, состоит ли строка только из N-кода (например "N0440")
    fn is_only_ncode(s: &str) -> bool {
        let s = s.trim();
        s.len() >= 2 && s.starts_with('N') && s[1..].chars().all(|c| c.is_ascii_digit())
    }
}

#[cfg(test)]
#[path = "formatter_tests.rs"]
mod tests;
