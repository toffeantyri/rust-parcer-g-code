//! Типы ошибок парсинга.

use crate::infrastructure::lexer::Span;
use thiserror::Error;

/// Ошибка парсинга G-кода.
#[derive(Debug, Clone, Error)]
pub enum ParseError {
    #[error("Unexpected token at {span:?}: expected {expected}, found {found:?}")]
    UnexpectedToken {
        span: Span,
        expected: String,
        found: String,
    },

    #[error("Unterminated parentheses at {span:?}")]
    UnterminatedParens { span: Span },

    #[error("Invalid number format at {span:?}")]
    InvalidNumber { span: Span },

    #[error("Unexpected end of input at {span:?}")]
    UnexpectedEof { span: Span },
}

/// Результат парсинга
#[allow(dead_code)]
pub type ParseResult<T> = Result<T, ParseError>;
