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
