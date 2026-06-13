//! Общие типы ошибок для всех слоёв приложения

use std::fmt;
use thiserror::Error;

/// Ошибка разбора G-кода
#[derive(Debug, Clone, Error)]
#[error("ParseError в позиции {position}: {message}")]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

/// Степень серьёзности ошибки валидации
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    /// Критическая ошибка — форматирование блокируется
    Error,
    /// Предупреждение — форматирование продолжается
    Warning,
}

/// Результат проверки одного оператора валидатором
#[derive(Debug, Clone, PartialEq, Error)]
pub struct ValidationMessage {
    pub severity: Severity,
    pub message: String,
    /// Номер строки в исходном файле (1-based). 0 — если строка неизвестна.
    pub line: usize,
}

impl ValidationMessage {
    pub fn error(line: usize, message: impl Into<String>) -> Self {
        ValidationMessage {
            severity: Severity::Error,
            message: message.into(),
            line,
        }
    }

    pub fn warning(line: usize, message: impl Into<String>) -> Self {
        ValidationMessage {
            severity: Severity::Warning,
            message: message.into(),
            line,
        }
    }
}

impl fmt::Display for ValidationMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let level = match self.severity {
            Severity::Error => "ОШИБКА",
            Severity::Warning => "ПРЕДУПРЕЖДЕНИЕ",
        };
        if self.line > 0 {
            write!(f, "{} [строка {}]: {}", level, self.line, self.message)
        } else {
            write!(f, "{}: {}", level, self.message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_with_line() {
        let msg = ValidationMessage::error(5, "ось X без значения");
        let s = msg.to_string();
        assert!(s.contains("ОШИБКА"));
        assert!(s.contains("[строка 5]"));
        assert!(s.contains("ось X без значения"));
    }

    #[test]
    fn test_warning_without_line() {
        let msg = ValidationMessage::warning(0, "проверьте оси");
        let s = msg.to_string();
        assert!(s.contains("ПРЕДУПРЕЖДЕНИЕ"));
        assert!(!s.contains("[строка"));
        assert!(s.contains("проверьте оси"));
    }

    #[test]
    fn test_error_with_line_zero() {
        let msg = ValidationMessage::error(0, "программа пуста");
        let s = msg.to_string();
        assert!(s.contains("ОШИБКА"));
        assert!(!s.contains("[строка"));
        assert_eq!(msg.severity, Severity::Error);
        assert_eq!(msg.line, 0);
    }

    #[test]
    fn test_parse_error_display() {
        let err = ParseError {
            message: "неизвестный токен".to_string(),
            position: 42,
        };
        let s = err.to_string();
        assert!(s.contains("ParseError"));
        assert!(s.contains("42"));
        assert!(s.contains("неизвестный токен"));
    }
}
