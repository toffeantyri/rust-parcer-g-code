//! Сырой токен (RawToken) от logos-лексера.
//! Распознаёт только синтаксис, не семантику.
//! Семантику (GCode, MCode, Axis, Word) собирает парсер.

use logos::Logos;

/// Позиция токена в исходной строке (байтовые индексы).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// Сырой токен — элементарная единица разбора G-кода,
/// распознаваемая регулярными выражениями logos.
///
/// Правила:
/// - `\n` НЕ в skip — граница блоков.
/// - `Number` БЕЗ знака (знак парсит парсер по контексту).
/// - `Unknown` — последний в enum (принцип длиннейшего совпадения).
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\r]+")]
pub enum RawToken<'a> {
    // --- Буквы и числа ---
    /// Одна буква A-Z / a-z
    #[regex(r"[A-Za-z]", |lex| lex.slice().chars().next().unwrap())]
    Letter(char),

    /// Число без знака (целое или с плавающей точкой)
    #[regex(r"\d+(\.\d*)?", |lex| lex.slice().parse::<f64>().unwrap())]
    Number(f64),

    // --- Скобки и разделители ---
    #[token("(")]
    ParenOpen,
    #[token(")")]
    ParenClose,
    #[token("[")]
    BracketOpen,
    #[token("]")]
    BracketClose,
    #[token(",")]
    Comma,

    // --- Операторы ---
    #[token("=")]
    Equals,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,
    #[token("<")]
    Less,
    #[token(">")]
    Greater,
    #[token("<=")]
    LessEq,
    #[token(">=")]
    GreaterEq,
    #[token("==")]
    EqEq,
    #[token("!=")]
    NotEq,

    // --- Комментарий (до конца строки, БЕЗ символа ;) ---
    #[regex(r";[^\n]*", |lex| &lex.slice()[1..])]
    Comment(&'a str),

    // --- Системные переменные Siemens ($TC_MPP6[9998,1]) ---
    #[regex(r"\$[A-Za-z_][A-Za-z0-9_]*(\[[^\]\n]*\])?", |lex| lex.slice())]
    SystemVar(&'a str),

    // --- Строковый литерал в кавычках ("Фрезеровка") ---
    #[regex(r#""[^"\n]*""#, |lex| &lex.slice()[1..lex.slice().len()-1])]
    String(&'a str),

    // --- Перевод строки ---
    #[token("\n")]
    NewLine,

    // --- Fallback: неизвестный символ (самый низкий приоритет) ---
    #[regex(r"[^ \t\r\n]", |lex| lex.slice().chars().next().unwrap(), priority = 1)]
    Unknown(char),
}

// ---------------------------------------------------------------------------
// Вспомогательная функция для лексинга
// ---------------------------------------------------------------------------

/// Лексер на базе logos: возвращает вектор (RawToken, Span).
pub fn tokenize_raw(input: &str) -> Vec<(RawToken<'_>, Span)> {
    let lex = RawToken::lexer(input);
    lex.spanned()
        .map(|(tok, span)| {
            (
                tok.unwrap_or(RawToken::Unknown('?')),
                Span {
                    start: span.start,
                    end: span.end,
                },
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number() {
        let tokens = tokenize_raw("123");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].0, RawToken::Number(123.0));
    }

    #[test]
    fn test_letter() {
        let tokens = tokenize_raw("X");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].0, RawToken::Letter('X'));
    }

    #[test]
    fn test_newline() {
        let tokens = tokenize_raw("\n");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].0, RawToken::NewLine);
    }

    #[test]
    fn test_comment() {
        let tokens = tokenize_raw("; test comment");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].0, RawToken::Comment(" test comment"));
    }

    #[test]
    fn test_system_var() {
        let tokens = tokenize_raw("$TC_MPP6[9998,1]");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].0, RawToken::SystemVar("$TC_MPP6[9998,1]"));
    }

    #[test]
    fn test_unknown() {
        let tokens = tokenize_raw("@");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].0, RawToken::Unknown('@'));
    }

    #[test]
    fn test_simple_gcode() {
        let tokens = tokenize_raw("G0");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].0, RawToken::Letter('G'));
        assert_eq!(tokens[1].0, RawToken::Number(0.0));
    }

    #[test]
    fn test_span_positions() {
        let tokens = tokenize_raw("X10.5");
        assert_eq!(tokens[0].1, Span { start: 0, end: 1 }); // X
        assert_eq!(tokens[1].1, Span { start: 1, end: 5 }); // 10.5
    }

    #[test]
    fn test_string_literal() {
        let tokens = tokenize_raw(r#""Hello""#);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].0, RawToken::String("Hello"));
    }

    #[test]
    fn test_operators() {
        let tokens = tokenize_raw("<= >= == != + - * /");
        let kinds: Vec<_> = tokens.iter().map(|t| t.0.clone()).collect();
        assert_eq!(
            kinds,
            vec![
                RawToken::LessEq,
                RawToken::GreaterEq,
                RawToken::EqEq,
                RawToken::NotEq,
                RawToken::Plus,
                RawToken::Minus,
                RawToken::Mul,
                RawToken::Div,
            ]
        );
    }
}
