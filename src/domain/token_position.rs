//! Тип для позиции токена (используется для подсветки)

use crate::domain::Token;

/// Токен с информацией о позиции в исходном тексте.
#[derive(Debug, Clone)]
pub struct TokenPosition {
    pub token: Token,
    /// Начало токена в исходной строке (байтовый индекс)
    pub start: usize,
    /// Конец токена (байтовый индекс, exclusive)
    pub end: usize,
}
