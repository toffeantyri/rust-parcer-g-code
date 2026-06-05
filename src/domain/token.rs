//! Доменные типы токенов, используемые лексером

/// Токен — элементарная единица разбора G-кода
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    GCode(i32),
    MCode(i32),
    Axis(String, f64),
    Number(f64),
    Comment(String),
    Eof,
    LParen,
    RParen,
    Semicolon,
    NewLine,
    Unknown(char),
}
