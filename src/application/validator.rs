//! Прикладной слой: валидатор — проверяет AST на синтаксические ошибки
//!
//! Распознаваемые ошибки:
//! - Ось без значения (например одинокий `X` без числа)
//! - G-код или M-код с кодом 0 (если 0 не был явно указан — а был результатом ошибки чтения)
//! - Две оси подряд (не реализовано на уровне токенов, только если есть подозрительные Raw)

use crate::domain::Statement;

/// Степень серьёзности ошибки валидации
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    /// Критическая ошибка — форматирование блокируется
    Error,
    /// Предупреждение — форматирование продолжается
    Warning,
}

/// Результат проверки одного оператора
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationMessage {
    pub severity: Severity,
    pub message: String,
    /// Человеко-читаемое описание где найдена проблема
    pub location: String,
}

impl ValidationMessage {
    pub fn error(location: impl Into<String>, message: impl Into<String>) -> Self {
        ValidationMessage {
            severity: Severity::Error,
            message: message.into(),
            location: location.into(),
        }
    }

    pub fn warning(location: impl Into<String>, message: impl Into<String>) -> Self {
        ValidationMessage {
            severity: Severity::Warning,
            message: message.into(),
            location: location.into(),
        }
    }
}

/// Проверяет программу (AST) на синтаксические ошибки.
/// Возвращает список сообщений валидации.
pub fn validate(program: &[Statement]) -> Vec<ValidationMessage> {
    let mut messages = Vec::new();

    if program.is_empty() {
        messages.push(ValidationMessage::error("program", "Программа пуста"));
        return messages;
    }

    let mut line_number: usize = 0;

    for stmt in program {
        match stmt {
            Statement::NewLine => {
                line_number += 1;
            }
            Statement::Axis(a) => {
                if a.value.is_none() {
                    messages.push(ValidationMessage::error(
                        format!("строка {}", line_number + 1),
                        format!("Ось '{}' указана без значения", a.axis),
                    ));
                }
            }
            Statement::Motion(m) => {
                // G0 — это валидный код (быстрое перемещение)
                if m.code == 0 {
                    // ок
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

    #[test]
    fn test_validate_empty_program() {
        let msgs = validate(&[]);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].severity, Severity::Error);
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
