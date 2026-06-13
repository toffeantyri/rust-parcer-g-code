//! Прикладной слой: форматтер — преобразует AST в отформатированный текст G-кода

use crate::domain::Statement;

/// Настройки форматирования G-кода
pub struct FormatConfig {
    pub indent_size: usize,
    pub use_spaces: bool,
    pub uppercase_codes: bool,
    pub decimal_places: usize,
    /// Шаг перенумерации кадров. 0 — не перенумеровывать
    pub renumber_step: u32,
    /// Пропускать пустые строки при перенумерации
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

/// Форматтер преобразует AST программы в строку G-кода
pub struct Formatter {
    config: FormatConfig,
}

impl Formatter {
    /// Создаёт форматтер с переданной конфигурацией
    pub fn new(config: FormatConfig) -> Self {
        Formatter { config }
    }

    /// Форматирует программу (вектор операторов) в строку
    pub fn format_program(&self, program: &[Statement]) -> String {
        let mut result = String::new();
        let mut line_parts: Vec<String> = Vec::new();
        let mut current_n: u32 = 0; // Текущий номер кадра (для перенумерации)
        let mut line_has_ncode = false;

        for stmt in program {
            match stmt {
                Statement::NewLine => {
                    // Завершение строки
                    if !line_parts.is_empty() {
                        let has_only_ncode = line_has_ncode && line_parts.len() == 1;

                        if self.config.renumber_step > 0
                            && has_only_ncode
                            && self.config.skip_empty_lines
                        {
                            // В строке только N-код — удаляем его (пустая строка)
                            line_parts.clear();
                            result.push('\n');
                        } else {
                            result.push_str(&self.format_line(&line_parts));
                            line_parts.clear();
                        }
                        line_has_ncode = false;
                    } else {
                        // Пустая строка (два NewLine подряд)
                        if self.config.renumber_step > 0 && !self.config.skip_empty_lines {
                            // Нумеруем пустую строку
                            current_n += self.config.renumber_step;
                            result.push_str(&format!("N{}\n", current_n));
                        } else {
                            result.push('\n');
                        }
                    }
                }
                Statement::NCode(_code) => {
                    if self.config.renumber_step > 0 {
                        // Заменяем на новый номер
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
                    // Если перенумерация включена, строка непустая и N-кода ещё нет — добавляем
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

    /// Форматирует один оператор в строку
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
                    format!("{}{:.prec$}", a.axis, v, prec = self.config.decimal_places)
                } else {
                    // Ось без значения — выводим как есть (будет ошибкой валидации)
                    a.axis.clone()
                }
            }
            Statement::Comment(c) => format!(";{}", c.text),
            Statement::Raw(raw) => raw.clone(),
            _ => stmt.to_string(),
        }
    }

    /// Склеивает части строки в одну строку с пробелами
    fn format_line(&self, parts: &[String]) -> String {
        format!("{}\n", parts.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::*;

    #[test]
    fn test_format_simple_program() {
        let program = vec![
            Statement::Motion(MotionStatement {
                code: 0,
                rapid: true,
            }),
            Statement::Axis(AxisStatement {
                axis: "X".to_string(),
                value: Some(10.0),
            }),
            Statement::Axis(AxisStatement {
                axis: "Y".to_string(),
                value: Some(20.0),
            }),
            Statement::NewLine,
            Statement::Motion(MotionStatement {
                code: 1,
                rapid: false,
            }),
            Statement::Axis(AxisStatement {
                axis: "Z".to_string(),
                value: Some(5.5),
            }),
            Statement::Axis(AxisStatement {
                axis: "F".to_string(),
                value: Some(100.0),
            }),
        ];

        let formatter = Formatter::new(FormatConfig::default());
        let result = formatter.format_program(&program);

        let expected = "G0 X10.00000 Y20.00000\nG1 Z5.50000 F100.00000\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_ncode() {
        // N-номер должен быть на одной строке с последующими операторами
        let program = vec![
            Statement::NCode(100),
            Statement::Motion(MotionStatement {
                code: 0,
                rapid: true,
            }),
            Statement::NewLine,
            Statement::NCode(105),
            Statement::Motion(MotionStatement {
                code: 1,
                rapid: false,
            }),
        ];

        let formatter = Formatter::new(FormatConfig::default());
        let result = formatter.format_program(&program);

        assert_eq!(result, "N0100 G0\nN0105 G1\n");
    }

    #[test]
    fn test_format_with_raw() {
        let program = vec![
            Statement::Motion(MotionStatement {
                code: 64,
                rapid: false,
            }),
            Statement::Raw("CFTCP".to_string()),
            Statement::NewLine,
            Statement::Raw("MODECHECK".to_string()),
            Statement::Comment(CommentStatement {
                text: "2".to_string(),
            }),
            Statement::NewLine,
            Statement::Raw("MAMILL".to_string()),
            Statement::NewLine,
            Statement::Motion(MotionStatement {
                code: 0,
                rapid: true,
            }),
        ];

        let formatter = Formatter::new(FormatConfig::default());
        let result = formatter.format_program(&program);

        // G64 и CFTCP — на одной строке, потом Raw-команды на отдельных строках
        assert!(result.contains("G64"));
        assert!(result.contains("CFTCP"));
    }

    #[test]
    fn test_format_empty_lines() {
        let program = vec![
            Statement::NewLine,
            Statement::NewLine,
            Statement::Motion(MotionStatement {
                code: 0,
                rapid: true,
            }),
        ];

        let formatter = Formatter::new(FormatConfig::default());
        let result = formatter.format_program(&program);

        assert_eq!(result, "\n\nG0\n");
    }

    #[test]
    fn test_renumber_step() {
        // Перенумерация с шагом 10
        let program = vec![
            Statement::NCode(100), // старый N-код будет заменён
            Statement::Motion(MotionStatement {
                code: 0,
                rapid: true,
            }),
            Statement::NewLine,
            // пустая строка
            Statement::NewLine,
            Statement::NCode(999), // заменится на следующий
            Statement::Motion(MotionStatement {
                code: 1,
                rapid: false,
            }),
        ];

        let config = FormatConfig {
            renumber_step: 10,
            skip_empty_lines: true,
            ..Default::default()
        };
        let formatter = Formatter::new(config);
        let result = formatter.format_program(&program);

        // N10, пустая строка (пропущена), N20
        assert_eq!(result, "N10 G0\n\nN20 G1\n");
    }

    #[test]
    fn test_renumber_skip_empty_false() {
        // Перенумерация без пропуска пустых строк
        let program = vec![
            Statement::NCode(1),
            Statement::Motion(MotionStatement {
                code: 0,
                rapid: true,
            }),
            Statement::NewLine,
            Statement::NewLine, // пустая строка
            Statement::NCode(2),
            Statement::Motion(MotionStatement {
                code: 1,
                rapid: false,
            }),
        ];

        let config = FormatConfig {
            renumber_step: 1,
            skip_empty_lines: false,
            ..Default::default()
        };
        let formatter = Formatter::new(config);
        let result = formatter.format_program(&program);

        // N1 G0, пустая строка получит N2, N3 G1
        assert_eq!(result, "N1 G0\nN2\nN3 G1\n");
    }

    #[test]
    fn test_renumber_adds_ncode_to_lines_without() {
        // Строки без N-кода тоже должны получить N-код при перенумерации
        let program = vec![
            Statement::Motion(MotionStatement {
                code: 0,
                rapid: true,
            }),
            Statement::Axis(AxisStatement {
                axis: "X".to_string(),
                value: Some(10.0),
            }),
            Statement::NewLine,
            Statement::Motion(MotionStatement {
                code: 1,
                rapid: false,
            }),
        ];

        let config = FormatConfig {
            renumber_step: 1,
            skip_empty_lines: true,
            ..Default::default()
        };
        let formatter = Formatter::new(config);
        let result = formatter.format_program(&program);

        // N1 G0 X10.00000\nN2 G1\n
        assert!(result.starts_with("N1 G0"));
        assert!(result.contains("\nN2 G1"));
    }

    #[test]
    fn test_renumber_removes_empty_ncode_lines() {
        // Если в строке только N-код и больше ничего — и skip_empty_lines=true — удаляем N-код
        let program = vec![
            Statement::NCode(100),
            Statement::NewLine,
            Statement::NCode(200),
            Statement::Motion(MotionStatement {
                code: 0,
                rapid: true,
            }),
        ];

        let config = FormatConfig {
            renumber_step: 10,
            skip_empty_lines: true,
            ..Default::default()
        };
        let formatter = Formatter::new(config);
        let result = formatter.format_program(&program);

        // Первая строка: был только N100 — удалён (пустая строка)
        // Вторая строка: N20 G0 (счётчик продолжился)
        assert_eq!(result, "\nN20 G0\n");
    }
}
