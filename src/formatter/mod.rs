// Форматировщик G-кода

use crate::ast::Statement;

pub struct Formatter {
    indent_size: usize,
    use_spaces: bool,
    uppercase_codes: bool,
}

impl Formatter {
    pub fn new() -> Self {
        Formatter {
            indent_size: 2,
            use_spaces: true,
            uppercase_codes: true,
        }
    }

    pub fn format_program(&self, program: &[Statement]) -> String {
        let mut result = String::new();
        let mut line_parts = Vec::new();

        for stmt in program {
            match stmt {
                Statement::NewLine => {
                    if !line_parts.is_empty() {
                        result.push_str(&self.format_line(&line_parts));
                        line_parts.clear();
                    }
                    result.push('\n');
                }
                Statement::Raw(raw) => {
                    // Сохраняем оригинальные конструкции без изменений
                    result.push_str(raw);
                }
                _ => {
                    line_parts.push(self.format_statement(stmt));
                }
            }
        }

        // Добавляем последнюю строку, если она не закончилась переводом строки
        if !line_parts.is_empty() {
            result.push_str(&self.format_line(&line_parts));
        }

        result
    }

    fn format_statement(&self, stmt: &Statement) -> String {
        match stmt {
            Statement::Motion(m) => {
                let code = if self.uppercase_codes { "G" } else { "g" };
                format!("{}{}", code, m.code)
            }
            Statement::Misc(m) => {
                let code = if self.uppercase_codes { "M" } else { "m" };
                format!("{}{}", code, m.code)
            }
            Statement::Axis(a) => {
                // Форматируем оси с фиксированным количеством знаков после запятой
                format!("{}{:.4}", a.axis, a.value)
            }
            Statement::Comment(c) => format!("({})", c.text),
            Statement::Raw(raw) => raw.clone(),
            _ => stmt.to_string(),
        }
    }

    fn format_line(&self, parts: &[String]) -> String {
        let line = parts.join(" ");
        // Можно добавить логику выравнивания здесь
        format!("{}\n", line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    #[test]
    fn test_format_simple_program() {
        let mut program = Vec::new();
        program.push(Statement::Motion(MotionStatement {
            code: 0,
            rapid: true,
        }));
        program.push(Statement::Axis(AxisStatement {
            axis: "X".to_string(),
            value: 10.0,
        }));
        program.push(Statement::Axis(AxisStatement {
            axis: "Y".to_string(),
            value: 20.0,
        }));
        program.push(Statement::NewLine);
        program.push(Statement::Motion(MotionStatement {
            code: 1,
            rapid: false,
        }));
        program.push(Statement::Axis(AxisStatement {
            axis: "Z".to_string(),
            value: 5.5,
        }));
        program.push(Statement::Axis(AxisStatement {
            axis: "F".to_string(),
            value: 100.0,
        }));

        let formatter = Formatter::new();
        let result = formatter.format_program(&program);

        let expected = "G0 X10.0000 Y20.0000\nG1 Z5.5000 F100.0000\n";
        assert_eq!(result, expected);
    }
}
