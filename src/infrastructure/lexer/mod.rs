//! Лексер G-кода на базе logos.
//!
//! Модульная структура:
//! - `raw_token` — атомарные токены (logos) + Span
//! - `parser` — ручной recursive descent парсер
//! - `keywords` — словарь ключевых слов
//! - `error` — типы ошибок

use crate::domain::{Lexer, Token};
use crate::infrastructure::lexer::keywords::KeywordDictionary;
use crate::infrastructure::lexer::parser::Parser;
use crate::infrastructure::lexer::raw_token::tokenize_raw;

pub use self::error::ParseError;
pub use self::raw_token::{RawToken, Span};

mod error;
mod keywords;
mod parser;
mod raw_token;

/// Стандартный лексер G-кода.
/// Реализует доменный трейт `Lexer`.
pub struct DefaultLexer {
    dictionary: KeywordDictionary,
}

impl DefaultLexer {
    pub fn new() -> Self {
        Self {
            dictionary: KeywordDictionary::siemens(),
        }
    }

    pub fn with_dictionary(dictionary: KeywordDictionary) -> Self {
        Self { dictionary }
    }
}

impl Default for DefaultLexer {
    fn default() -> Self {
        Self::new()
    }
}

impl Lexer for DefaultLexer {
    fn tokenize(&self, input: &str) -> Vec<Token> {
        tokenize_with_dict(input, &self.dictionary)
    }
}

/// Основная функция токенизации (обратно совместима с `infrastructure::lexer::tokenize`).
pub fn tokenize(input: &str) -> Vec<Token> {
    tokenize_with_dict(input, &KeywordDictionary::siemens())
}

/// Токенизация с указанным словарём ключевых слов.
pub fn tokenize_with_dict(input: &str, dictionary: &KeywordDictionary) -> Vec<Token> {
    let raw_tokens = tokenize_raw(input);
    let mut parser = Parser::new(raw_tokens, dictionary, input);
    parser.parse_program()
}

/// Токен с позицией в исходном тексте.
#[derive(Debug, Clone)]
pub struct TokenWithPosition {
    pub token: Token,
    pub start: usize,
    pub end: usize,
}

/// Токенизация с возвратом позиций.
/// Используется для подсветки синтаксиса в GUI.
pub fn tokenize_with_positions(input: &str) -> Vec<TokenWithPosition> {
    let raw_tokens = tokenize_raw(input);
    let dict = KeywordDictionary::siemens();

    let mut parser = Parser::new(raw_tokens, &dict, input);
    let tokens = parser.parse_program_spanned();

    tokens
        .into_iter()
        .map(|(token, span)| TokenWithPosition {
            token,
            start: span.start,
            end: span.end,
        })
        .collect()
}
