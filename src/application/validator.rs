//! Прикладной слой: валидатор — проверяет AST на синтаксические ошибки
//!
//! Проверки:
//! - Ось без значения (например одинокий `X` без числа)
//! - Пустая программа

use crate::domain::Statement;
use crate::shared::ValidationMessage;

/// Проверяет программу (AST) на синтаксические ошибки.
/// Возвращает список сообщений валидации с номерами строк.
pub fn validate(program: &[Statement]) -> Vec<ValidationMessage> {
    let mut messages = Vec::new();

    if program.is_empty() {
        messages.push(ValidationMessage::error(0, "Программа пуста"));
        return messages;
    }

    // Номер текущей строки (1-based)
    let mut line: usize = 1;

    for stmt in program {
        match stmt {
            Statement::NewLine => {
                line += 1;
            }
            Statement::Axis(a) => {
                if a.value.is_none() {
                    messages.push(ValidationMessage::error(
                        line,
                        format!("Ось '{}' указана без значения", a.axis),
                    ));
                }
            }
            _ => {}
        }
    }

    messages
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::*;
    use crate::shared::Severity;

    #[test]
    fn test_validate_empty_program() {
        let msgs = validate(&[]);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].severity, Severity::Error);
        assert_eq!(msgs[0].line, 0);
    }

    #[test]
    fn test_validate_axis_without_value() {
        let program = vec![Statement::Axis(AxisStatement {
            axis: "X".to_string(),
            value: None,
        })];
        let msgs = validate(&program);
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].message.contains("X"));
        // Первая строка — line=1
        assert_eq!(msgs[0].line, 1);
    }

    #[test]
    fn test_validate_axis_without_value_on_line_3() {
        let program = vec![
            Statement::NewLine,
            Statement::NewLine,
            Statement::Axis(AxisStatement {
                axis: "Y".to_string(),
                value: None,
            }),
        ];
        let msgs = validate(&program);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].line, 3);
    }

    #[test]
    fn test_validate_ok_program() {
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
        ];
        let msgs = validate(&program);
        assert!(msgs.is_empty());
    }
}
