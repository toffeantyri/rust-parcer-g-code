//! Рекурсивный парсер: собирает семантические `Token` из атомарных `RawToken`.
//!
//! Парсер принимает вектор `(RawToken, Span)` от logos-лексера,
//! распознаёт конструкции G-кода (G-коды, M-коды, оси, R-параметры,
//! ключевые слова) и возвращает доменный `Vec<Token>`.

use crate::domain::Token;
use crate::infrastructure::lexer::keywords::KeywordDictionary;
use crate::infrastructure::lexer::raw_token::{RawToken, Span};

/// Парсер G-кода.
pub struct Parser<'a> {
    tokens: Vec<(RawToken<'a>, Span)>,
    pos: usize,
    dictionary: &'a KeywordDictionary,
    input: &'a str,
}

impl<'a> Parser<'a> {
    /// Создаёт новый парсер из вектора токенов logos.
    pub fn new(
        tokens: Vec<(RawToken<'a>, Span)>,
        dictionary: &'a KeywordDictionary,
        input: &'a str,
    ) -> Self {
        Self {
            tokens,
            pos: 0,
            dictionary,
            input,
        }
    }

    // ── Вспомогательные методы ──────────────────────────────────────

    /// Текущий сырой токен (клонированный, без borrow на self).
    fn current(&self) -> Option<(RawToken<'a>, Span)> {
        self.tokens.get(self.pos).cloned()
    }

    /// Продвигает позицию на 1 и возвращает предыдущий токен.
    fn advance(&mut self) -> Option<(RawToken<'a>, Span)> {
        let tok = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        tok
    }

    /// Следующий токен (без продвижения).
    fn peek_next(&self) -> Option<(RawToken<'a>, Span)> {
        self.tokens.get(self.pos + 1).cloned()
    }

    /// Проверяет, есть ли ещё токены.
    #[allow(dead_code)]
    fn has_more(&self) -> bool {
        self.pos < self.tokens.len()
    }

    #[allow(dead_code)]
    fn skip_newlines(&mut self) {
        while let Some((RawToken::NewLine, _)) = self.current() {
            self.advance();
        }
    }

    /// Пропускает все токены до конца строки (NewLine или Eof),
    /// НО останавливается перед Comment (комментарий остаётся отдельным токеном).
    fn skip_to_end_of_line(&mut self) {
        loop {
            match self.current() {
                None | Some((RawToken::NewLine, _)) => return,
                Some((RawToken::Comment(_), _)) => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    /// Берёт срез исходной строки по Span.
    fn slice(&self, span: &Span) -> &'a str {
        &self.input[span.start..span.end]
    }

    // ── Основной метод парсинга ─────────────────────────────────────

    /// Парсит всю программу и возвращает плоский `Vec<Token>`.
    /// `Eof` в публичный API не попадает.
    pub fn parse_program(&mut self) -> Vec<Token> {
        let mut result = Vec::new();
        loop {
            if self.current().is_none() {
                break;
            }
            let tokens = self.parse_block();
            result.extend(tokens);
        }
        result
    }

    // ── Парсинг блока (строка программы) ────────────────────────────

    /// Парсит блок токенов до NewLine или Eof.
    fn parse_block(&mut self) -> Vec<Token> {
        let mut result = Vec::new();

        loop {
            match self.current() {
                None => break,
                Some((RawToken::NewLine, _)) => {
                    self.advance();
                    result.push(Token::NewLine);
                    break;
                }
                Some((RawToken::Unknown(ch), _)) => {
                    self.advance();
                    result.push(Token::Unknown(ch));
                    continue;
                }
                Some((RawToken::Comment(_), _)) => {
                    // Комментарий может быть на отдельной строке
                    result.push(self.parse_comment());
                    continue;
                }
                Some((RawToken::SystemVar(s), _)) => {
                    self.advance();
                    result.push(Token::Word(s.to_string()));
                    continue;
                }
                _ => {}
            }

            // Пробуем распознать конструкцию
            if let Some(tok) = self.try_parse_ncode() {
                result.push(tok);
                continue;
            }
            if let Some(tok) = self.try_parse_gcode() {
                result.push(tok);
                continue;
            }
            if let Some(tok) = self.try_parse_mcode() {
                result.push(tok);
                continue;
            }
            if let Some(tok) = self.try_parse_speed() {
                result.push(tok);
                continue;
            }
            if let Some(tok) = self.try_parse_r_parameter() {
                result.push(tok);
                continue;
            }
            if let Some(tok) = self.try_parse_axis() {
                result.push(tok);
                continue;
            }
            if let Some(tok) = self.try_parse_word_or_letter() {
                result.push(tok);
                continue;
            }

            // Ничего не распознано — продвигаемся на один токен
            if let Some((RawToken::Number(n), _)) = self.advance() {
                result.push(Token::Number(n));
            }
        }

        result
    }

    // ── Парсинг комментария ─────────────────────────────────────────

    fn parse_comment(&mut self) -> Token {
        match self.advance() {
            Some((RawToken::Comment(text), _)) => Token::Comment(text.to_string()),
            _ => Token::Comment(String::new()),
        }
    }

    // ── G-код ───────────────────────────────────────────────────────

    fn try_parse_gcode(&mut self) -> Option<Token> {
        let save = self.pos;
        match self.current() {
            Some((RawToken::Letter('G') | RawToken::Letter('g'), _)) => {
                self.advance();
            }
            _ => return None,
        }

        // После G должно быть число (может с минусом)
        let num = match self.parse_signed_number() {
            Some(n) => n,
            None => {
                self.pos = save;
                return None;
            }
        };
        Some(Token::GCode(num as i32))
    }

    // ── M-код ───────────────────────────────────────────────────────

    fn try_parse_mcode(&mut self) -> Option<Token> {
        let save = self.pos;
        match self.current() {
            Some((RawToken::Letter('M') | RawToken::Letter('m'), _)) => {
                self.advance();
            }
            _ => return None,
        }

        let num = match self.parse_signed_number() {
            Some(n) => n,
            None => {
                self.pos = save;
                return None;
            }
        };
        Some(Token::MCode(num as i32))
    }

    // ── N-код (номер кадра) ─────────────────────────────────────────

    fn try_parse_ncode(&mut self) -> Option<Token> {
        let save = self.pos;
        match self.current() {
            Some((RawToken::Letter('N') | RawToken::Letter('n'), _)) => {
                self.advance();
            }
            _ => return None,
        }

        let num = match self.parse_number_only() {
            Some(n) => n,
            None => {
                self.pos = save;
                return None;
            }
        };
        Some(Token::NCode(num as i32))
    }

    // ── Speed (скорость шпинделя S или подача F) ──────────────────

    fn try_parse_speed(&mut self) -> Option<Token> {
        let save = self.pos;
        let prefix = match self.current() {
            Some((RawToken::Letter('S') | RawToken::Letter('s'), _)) => {
                self.advance();
                'S'
            }
            Some((RawToken::Letter('F') | RawToken::Letter('f'), _)) => {
                self.advance();
                'F'
            }
            _ => return None,
        };

        // Определяем подтип Speed: S1=1000, SS1=500 или просто S1000
        // F100, F200 — подача
        let mut raw = prefix.to_string();

        // Для S: может быть вторая S (SS...)
        if prefix == 'S' {
            if let Some((RawToken::Letter('S'), _)) = self.current() {
                self.advance();
                raw.push('S');
            }
        }

        // Собираем число
        let num = match self.parse_signed_number() {
            Some(n) => n,
            None => {
                self.pos = save;
                return None;
            }
        };
        raw.push_str(&format_number(num));

        // Если встретили `=`, читаем значение ступени (только для S)
        if prefix == 'S' {
            if let Some((RawToken::Equals, _)) = self.current() {
                self.advance();
                raw.push('=');
                let step_val = self.parse_signed_number()?;
                raw.push_str(&format_number(step_val));
            }
        }

        Some(Token::Speed(raw))
    }

    // ── R-параметр ─────────────────────────────────────────────────

    fn try_parse_r_parameter(&mut self) -> Option<Token> {
        let save = self.pos;
        match self.current() {
            Some((RawToken::Letter('R') | RawToken::Letter('r'), _)) => {
                self.advance();
            }
            _ => return None,
        };

        // После R должны быть цифры (R50, R101).
        // Пробел между R и числом допускается (R 100 = 5)
        // — это стандартный Siemens-формат R-параметра.
        match self.current() {
            Some((RawToken::Number(n), _)) => {
                let n_int = n as i32;
                if n_int as f64 != n || n_int <= 0 {
                    self.pos = save;
                    return None;
                }
                self.advance();
                let mut raw = format!("R{}", n_int);

                // Может быть = выражение
                if let Some((RawToken::Equals, _)) = self.current() {
                    self.advance();
                    raw.push('=');
                    // Читаем выражение до конца строки или до ;
                    raw.push_str(&self.collect_expression());
                }

                Some(Token::RParameter(raw))
            }
            _ => {
                // R без цифр — это не R-параметр, а слово
                self.pos = save;
                None
            }
        }
    }

    // ── Ось ─────────────────────────────────────────────────────────

    fn try_parse_axis(&mut self) -> Option<Token> {
        let letter = match self.current() {
            Some((RawToken::Letter(l), _)) if AXIS_LETTERS.contains(&l.to_ascii_uppercase()) => {
                let l = l.to_ascii_uppercase();
                // Проверяем, что следующая буква НЕ часть слова
                // (чтобы не спутать W в WHILE с осью W)
                if let Some((RawToken::Letter(_), _)) = self.peek_next() {
                    return None;
                }
                self.advance();
                l
            }
            _ => return None,
        };

        // Следующий токен определяет тип: ось с числом, ось с выражением, или просто ось
        match self.current() {
            // X = 160+10 — AxisExpr
            Some((RawToken::Equals, _)) => {
                self.advance();
                // Выражение: читаем всё до конца блока
                let expr_span = self.collect_expression_span();
                let expr = self.slice(&expr_span).trim().to_string();
                Some(Token::AxisExpr(format!("{}", letter), expr))
            }
            // X-10.5, X10, X — ось с числом или без
            _ => {
                let (value, decimal_places) = self.parse_axis_value();
                Some(Token::Axis(format!("{}", letter), value, decimal_places))
            }
        }
    }

    /// Собирает ось со значением: `X10.5`, `X-10.5`, `X` (без значения).
    fn parse_axis_value(&mut self) -> (Option<f64>, Option<usize>) {
        // Может быть минус
        let neg = if let Some((RawToken::Minus, _)) = self.current() {
            self.advance();
            true
        } else {
            false
        };

        match self.current() {
            Some((RawToken::Number(val), span)) => {
                let dec = count_decimal_places(self.slice(&span));
                self.advance();
                if neg {
                    (Some(-val), dec)
                } else {
                    (Some(val), dec)
                }
            }
            _ => {
                if neg {
                    // Был минус без числа — невалидно
                    (None, None)
                } else {
                    (None, None)
                }
            }
        }
    }

    // ── Слово / буква (многосимвольные слова, ключевые слова потока) ──

    fn try_parse_word_or_letter(&mut self) -> Option<Token> {
        let save = self.pos;

        // Особый случай: скобочное выражение без слова перед ним
        // (Rapid move) — это тоже Word
        if let Some((RawToken::ParenOpen, _)) = self.current() {
            self.advance(); // продвигаем открывающую скобку
            let args = self.collect_paren_args();
            return Some(Token::Word(format!("({})", args)));
        }

        // Должна быть буква для обычного слова
        match self.current() {
            Some((RawToken::Letter(_), _)) => {}
            _ => return None,
        }

        // Собираем слово
        let word = self.collect_word();

        if word.is_empty() {
            return None;
        }

        // Проверяем по словарю
        if self.dictionary.is_flow_control(&word) {
            // Ключевое слово потока: захватываем всё до конца строки
            let word_span = self.find_word_span(save);
            let full_text = self.slice(&word_span).trim().to_string();
            self.skip_to_end_of_line();
            return Some(Token::Word(full_text));
        }

        if self.dictionary.is_system_command(&word) || self.dictionary.is_miscellaneous(&word) {
            // Системная команда или вспомогательное слово:
            // парсится как обычное слово со скобочными аргументами
            // (не захватывает всю строку)
            return self.finish_word(word);
        }

        // Обычное слово: может иметь скобочные аргументы,
        // или = выражение (R100=5 после пробелов)
        self.finish_word(word)
    }

    /// Завершает парсинг обычного слова: проверяет скобки или = выражение.
    fn finish_word(&mut self, word: String) -> Option<Token> {
        // Обычное слово: может иметь скобочные аргументы,
        // или = выражение
        if let Some((RawToken::ParenOpen, _)) = self.current() {
            self.advance();
            let args = self.collect_paren_args();
            Some(Token::Word(format!("{}({})", word, args)))
        } else if let Some((RawToken::Equals, _)) = self.current() {
            self.advance();
            let expr_span = self.collect_expression_span();
            let expr = self.slice(&expr_span).trim().to_string();
            Some(Token::Word(format!("{}={}", word, expr)))
        } else {
            Some(Token::Word(word))
        }
    }

    // ── Вспомогательные методы для сбора выражений ─────────────────

    /// Собирает слово из последовательных Letter (и Number, если они примыкают вплотную
    /// и слово является известным ключевым).
    /// Для неизвестных слов числа не включаются (отдельный `Number` токен).
    fn collect_word(&mut self) -> String {
        let mut word = String::new();
        let mut last_end: Option<usize> = None;
        while let Some((RawToken::Letter(c), span)) = self.current() {
            if let Some(end) = last_end {
                if end != span.start {
                    break;
                }
            }
            word.push(c.to_ascii_uppercase());
            last_end = Some(span.end);
            self.advance();
        }
        word
    }

    /// Находит Span от позиции save до конца строки (до NewLine или Eof),
    /// но останавливается перед Comment.
    fn find_word_span(&self, save: usize) -> Span {
        let start = self.tokens.get(save).map(|t| t.1.start).unwrap_or(0);
        // Ищем конец: NewLine, Eof или Comment
        let mut end = start;
        for i in save..self.tokens.len() {
            match &self.tokens[i].0 {
                RawToken::NewLine => break,
                RawToken::Comment(_) => break,
                _ => {
                    end = self.tokens[i].1.end;
                }
            }
        }
        Span { start, end }
    }

    /// Собирает аргументы в скобках с учётом вложенности.
    /// Вызывается ПОСЛЕ того, как открывающая скобка уже потреблена (advance).
    fn collect_paren_args(&mut self) -> String {
        let mut depth = 1usize;
        let mut args = String::new();
        let mut last_end: Option<usize> = None;

        loop {
            match self.current() {
                None | Some((RawToken::NewLine, _)) => break,
                Some((RawToken::ParenOpen, span)) => {
                    self.add_paren_space(&mut args, last_end, span.start);
                    self.advance();
                    depth += 1;
                    if depth > 1 {
                        args.push('(');
                    }
                    last_end = Some(span.end);
                }
                Some((RawToken::ParenClose, span)) => {
                    self.advance();
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    self.add_paren_space(&mut args, last_end, span.start);
                    args.push(')');
                    last_end = Some(span.end);
                }
                Some((raw, span)) => {
                    self.add_paren_space(&mut args, last_end, span.start);
                    self.advance();
                    match raw {
                        RawToken::Number(n) => args.push_str(&format_number(n)),
                        RawToken::Letter(c) => args.push(c),
                        RawToken::Comma => args.push(','),
                        RawToken::Minus => args.push('-'),
                        RawToken::Plus => args.push('+'),
                        RawToken::Mul => args.push('*'),
                        RawToken::Div => args.push('/'),
                        RawToken::Equals => args.push('='),
                        RawToken::String(s) => {
                            args.push('"');
                            args.push_str(s);
                            args.push('"');
                        }
                        _ => {}
                    }
                    last_end = Some(span.end);
                }
            }
        }

        // Обрезаем лишние пробелы
        args.trim().to_string()
    }

    /// Добавляет пробел в скобочное выражение, если между токенами был разрыв.
    fn add_paren_space(&self, args: &mut String, last_end: Option<usize>, current_start: usize) {
        if let Some(prev_end) = last_end {
            if prev_end != current_start {
                args.push(' ');
            }
        }
    }

    /// Собирает выражение (значение после `=`) в R-параметре.
    fn collect_expression(&mut self) -> String {
        let mut expr = String::new();
        loop {
            match self.current() {
                None | Some((RawToken::NewLine, _)) | Some((RawToken::Comment(_), _)) => break,
                Some((raw, _)) => {
                    self.advance();
                    match raw {
                        RawToken::Number(n) => expr.push_str(&format_number(n)),
                        RawToken::Letter(c) => expr.push(c.to_ascii_uppercase()),
                        RawToken::Minus => expr.push('-'),
                        RawToken::Plus => expr.push('+'),
                        RawToken::Mul => expr.push('*'),
                        RawToken::Div => expr.push('/'),
                        RawToken::Equals => expr.push('='),
                        RawToken::ParenOpen => expr.push('('),
                        RawToken::ParenClose => expr.push(')'),
                        RawToken::BracketOpen => expr.push('['),
                        RawToken::BracketClose => expr.push(']'),
                        RawToken::Comma => expr.push(','),
                        RawToken::Less => expr.push('<'),
                        RawToken::Greater => expr.push('>'),
                        RawToken::SystemVar(s) => expr.push_str(s),
                        _ => {}
                    }
                }
            }
        }
        expr.trim().to_string()
    }

    /// Собирает Span выражения (после `=`) для оси.
    /// Останавливается на пробеле (проверка по Span) или NewLine/Comment.
    fn collect_expression_span(&mut self) -> Span {
        let start = self.tokens.get(self.pos).map(|t| t.1.start).unwrap_or(0);
        let mut end = start;
        let mut last_end: Option<usize> = None;
        loop {
            match self.current() {
                None | Some((RawToken::NewLine, _)) | Some((RawToken::Comment(_), _)) => break,
                Some((_, span)) => {
                    // Если между токенами есть пробел — останавливаемся
                    if let Some(prev_end) = last_end {
                        if prev_end != span.start {
                            break;
                        }
                    }
                    end = span.end;
                    last_end = Some(span.end);
                    self.advance();
                }
            }
        }
        Span { start, end }
    }

    // ── Числовые хелперы ────────────────────────────────────────────

    /// Парсит число со знаком (опциональный минус + Number).
    fn parse_signed_number(&mut self) -> Option<f64> {
        let neg = if let Some((RawToken::Minus, _)) = self.current() {
            self.advance();
            true
        } else {
            false
        };

        match self.advance() {
            Some((RawToken::Number(n), _)) => {
                if neg {
                    Some(-n)
                } else {
                    Some(n)
                }
            }
            _ => None,
        }
    }

    /// Парсит только положительное число (без знака).
    fn parse_number_only(&mut self) -> Option<f64> {
        match self.advance() {
            Some((RawToken::Number(n), _)) => Some(n),
            _ => None,
        }
    }
}

// ── Константы и утилиты ──────────────────────────────────────────────

/// Буквы осей (стандартные оси G-кода)
const AXIS_LETTERS: &[char] = &['X', 'Y', 'Z', 'A', 'B', 'C', 'U', 'V', 'W', 'I', 'J', 'K'];

/// Подсчитывает количество знаков после запятой в строковом представлении числа.
fn count_decimal_places(s: &str) -> Option<usize> {
    if let Some(dot_pos) = s.find('.') {
        let after_dot = &s[dot_pos + 1..];
        if after_dot.is_empty() {
            Some(0)
        } else {
            Some(after_dot.len())
        }
    } else {
        None
    }
}

/// Форматирует число в строку без лишних знаков (целые без точки).
fn format_number(n: f64) -> String {
    if n == n.trunc() && n.is_finite() {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::lexer::raw_token::tokenize_raw;

    fn parse(input: &str) -> Vec<Token> {
        let dict = KeywordDictionary::siemens();
        let raw_tokens = tokenize_raw(input);
        let mut parser = Parser::new(raw_tokens, &dict, input);
        parser.parse_program()
    }

    fn tokenize(input: &str) -> Vec<Token> {
        let dict = KeywordDictionary::siemens();
        let raw_tokens = tokenize_raw(input);
        let mut parser = Parser::new(raw_tokens, &dict, input);
        parser.parse_program()
    }

    #[test]
    fn test_gcode_simple() {
        let tokens = tokenize("G0");
        assert_eq!(tokens, vec![Token::GCode(0)]);
    }

    #[test]
    fn test_gcode_with_number() {
        let tokens = tokenize("G01");
        assert_eq!(tokens, vec![Token::GCode(1)]);
    }

    #[test]
    fn test_gcode_lowercase() {
        let tokens = tokenize("g1");
        assert_eq!(tokens, vec![Token::GCode(1)]);
    }

    #[test]
    fn test_mcode_simple() {
        let tokens = tokenize("M3");
        assert_eq!(tokens, vec![Token::MCode(3)]);
    }

    #[test]
    fn test_mcode_lowercase() {
        let tokens = tokenize("m30");
        assert_eq!(tokens, vec![Token::MCode(30)]);
    }

    #[test]
    fn test_ncode() {
        let tokens = tokenize("N100");
        assert_eq!(tokens, vec![Token::NCode(100)]);
    }

    #[test]
    fn test_axis_with_value() {
        let tokens = tokenize("X10.5");
        assert_eq!(
            tokens,
            vec![Token::Axis("X".to_string(), Some(10.5), Some(1))]
        );
    }

    #[test]
    fn test_axis_without_value() {
        let tokens = tokenize("X");
        assert_eq!(tokens, vec![Token::Axis("X".to_string(), None, None)]);
    }

    #[test]
    fn test_axis_negative() {
        let tokens = tokenize("X-10.5");
        assert_eq!(
            tokens,
            vec![Token::Axis("X".to_string(), Some(-10.5), Some(1))]
        );
    }

    #[test]
    fn test_axis_expr() {
        let tokens = tokenize("Z=71.304");
        assert_eq!(
            tokens,
            vec![Token::AxisExpr("Z".to_string(), "71.304".to_string())]
        );
    }

    #[test]
    fn test_axis_negative_without_expr() {
        let tokens = tokenize("Y=-0.03");
        assert_eq!(
            tokens,
            vec![Token::AxisExpr("Y".to_string(), "-0.03".to_string()),]
        );
    }

    // ── Новые тесты Шаг 2 ───────────────────────────────────────────

    #[test]
    fn test_while_keyword() {
        let tokens = tokenize("WHILE (R101 < 10)");
        assert_eq!(tokens, vec![Token::Word("WHILE (R101 < 10)".to_string())]);
    }

    #[test]
    fn test_if_keyword() {
        let tokens = tokenize("IF (R101 == 0)");
        assert_eq!(tokens, vec![Token::Word("IF (R101 == 0)".to_string())]);
    }

    #[test]
    fn test_endwhile() {
        let tokens = tokenize("ENDWHILE");
        assert_eq!(tokens, vec![Token::Word("ENDWHILE".to_string())]);
    }

    #[test]
    fn test_system_command_with_parens() {
        let tokens = tokenize("MODECHECK(2)");
        assert_eq!(tokens, vec![Token::Word("MODECHECK(2)".to_string())]);
    }

    #[test]
    fn test_system_command_with_nested_parens() {
        let tokens = tokenize("MODECHECK(FOO(1,2))");
        assert_eq!(tokens, vec![Token::Word("MODECHECK(FOO(1,2))".to_string())]);
    }

    #[test]
    fn test_msg_command() {
        let tokens = tokenize(r###"MSG("Фрезеровка площадки")"###);
        assert_eq!(
            tokens,
            vec![Token::Word(r###"MSG("Фрезеровка площадки")"###.to_string())]
        );
    }

    #[test]
    fn test_r_parameter_with_spaces() {
        let tokens = tokenize("R 100 = 5");
        assert_eq!(tokens, vec![Token::RParameter("R100=5".to_string())]);
    }

    #[test]
    fn test_r_parameter_expression() {
        let tokens = tokenize("R101=R101+1");
        assert_eq!(tokens, vec![Token::RParameter("R101=R101+1".to_string())]);
    }

    #[test]
    fn test_empty_input() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_only_comment() {
        let tokens = tokenize("; test");
        assert_eq!(tokens, vec![Token::Comment(" test".to_string())]);
    }

    #[test]
    fn test_unknown_chars() {
        let tokens = tokenize("@#");
        assert_eq!(tokens, vec![Token::Unknown('@'), Token::Unknown('#')]);
    }

    #[test]
    fn test_mixed_case() {
        let tokens = tokenize("g1 x10");
        assert_eq!(
            tokens,
            vec![
                Token::GCode(1),
                Token::Axis("X".to_string(), Some(10.0), None),
            ]
        );
    }

    #[test]
    fn test_speed_simple() {
        let tokens = tokenize("S1000");
        assert_eq!(tokens, vec![Token::Speed("S1000".to_string())]);
    }

    #[test]
    fn test_speed_with_step() {
        let tokens = tokenize("S1=1000");
        assert_eq!(tokens, vec![Token::Speed("S1=1000".to_string())]);
    }

    #[test]
    fn test_speed_ss() {
        let tokens = tokenize("SS1=500");
        assert_eq!(tokens, vec![Token::Speed("SS1=500".to_string())]);
    }

    #[test]
    fn test_system_var_as_word() {
        let tokens = tokenize("$TC_MPP6[9998,1]");
        assert_eq!(tokens, vec![Token::Word("$TC_MPP6[9998,1]".to_string())]);
    }

    #[test]
    fn test_multiple_newlines() {
        let tokens = tokenize("G0\n\nG1");
        assert_eq!(
            tokens,
            vec![
                Token::GCode(0),
                Token::NewLine,
                Token::NewLine,
                Token::GCode(1),
            ]
        );
    }

    #[test]
    fn test_system_command_with_spaces_before_paren() {
        let tokens = tokenize("CFTCP");
        assert_eq!(tokens, vec![Token::Word("CFTCP".to_string())]);
    }

    #[test]
    fn test_word_without_args() {
        let tokens = tokenize("MAMILL");
        assert_eq!(tokens, vec![Token::Word("MAMILL".to_string())]);
    }
}
