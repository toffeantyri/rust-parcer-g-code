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
