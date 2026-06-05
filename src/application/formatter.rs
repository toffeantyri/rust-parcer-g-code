//! Прикладной слой: форматтер — преобразует AST в отформатированный текст G-кода

use crate::domain::Statement;

/// Настройки форматирования G-кода
pub struct FormatConfig {
    pub indent_size: usize,
    pub use_spaces: bool,
    pub uppercase_codes: bool,
    pub decimal_places: usize,
}

impl Default for FormatConfig {
    fn default() -> Self {
        FormatConfig {
            indent_size: 2,
            use_spaces: true,
            uppercase_codes: true,
            decimal_places: 4,
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

        for stmt in program {
            match stmt {
                Statement::NewLine => {
                    if !line_parts.is_empty() {
                        result.push_str(&self.format_line(&line_parts));
                        line_parts.clear();
                    } else {
                        // Пустая строка (два NewLine подряд)
                        result.push('\n');
                    }
                }
                Statement::NCode(_code) => {
                    // N-номер начинает новую строку: сбрасываем текущую, начинаем новую с N
                    if !line_parts.is_empty() {
                        result.push_str(&self.format_line(&line_parts));
                        line_parts.clear();
                    }
                    line_parts.push(self.format_statement(stmt));
                }
                Statement::Raw(raw) => {
                    // Если в line_parts есть незавершённые операторы — сначала завершаем строку
                    if !line_parts.is_empty() {
                        result.push_str(&self.format_line(&line_parts));
                        line_parts.clear();
                    }
                    // Raw-оператор может быть на своей строке, не смешиваем
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
                format!(
                    "{}{:.prec$}",
                    a.axis,
                    a.value,
                    prec = self.config.decimal_places
                )
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
                value: 10.0,
            }),
            Statement::Axis(AxisStatement {
                axis: "Y".to_string(),
                value: 20.0,
            }),
            Statement::NewLine,
            Statement::Motion(MotionStatement {
                code: 1,
                rapid: false,
            }),
            Statement::Axis(AxisStatement {
                axis: "Z".to_string(),
                value: 5.5,
            }),
            Statement::Axis(AxisStatement {
                axis: "F".to_string(),
                value: 100.0,
            }),
        ];

        let formatter = Formatter::new(FormatConfig::default());
        let result = formatter.format_program(&program);

        let expected = "G0 X10.0000 Y20.0000\nG1 Z5.5000 F100.0000\n";
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
}
