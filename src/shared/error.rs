//! Общие типы ошибок для всех слоёв приложения

use std::fmt;

/// Ошибка разбора G-кода
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ParseError at {}: {}", self.position, self.message)
    }
}

/// Результат валидации AST
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn new() -> Self {
        ValidationResult {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Есть ли критические ошибки (блокируют форматирование)
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Есть ли предупреждения
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

/// Ошибка валидации G-кода
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
    pub position: Option<usize>,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.position {
            Some(pos) => write!(f, "at {}: {}", pos, self.message),
            None => write!(f, "{}", self.message),
        }
    }
}
