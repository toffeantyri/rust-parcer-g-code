//! Инфраструктурный слой: лексер — преобразует текст в токены
//!
//! Лексер зависит от доменных типов Token, но не содержит бизнес-логики.

use crate::domain::Token;

/// Лексер для разбора текста программы G-кода в последовательность токенов
pub struct Lexer {
    input: String,
    position: usize,
    read_position: usize,
    ch: char,
}

impl Lexer {
    /// Создаёт новый лексер для переданного текста
    pub fn new(input: String) -> Self {
        let mut lexer = Lexer {
            input,
            position: 0,
            read_position: 0,
            ch: '\0',
        };
        lexer.read_char();
        lexer
    }

    /// Считывает следующий символ, обновляя позиции
    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0';
        } else {
            self.ch = self.input[self.read_position..]
                .chars()
                .next()
                .unwrap_or('\0');
            self.position = self.read_position;
            self.read_position += self.ch.len_utf8();
        }
    }

    /// Пропускает пробельные символы (кроме перевода строки)
    fn skip_whitespace(&mut self) {
        while self.ch.is_whitespace() && self.ch != '\n' {
            self.read_char();
        }
    }

    /// Возвращает следующий токен из входного потока
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let ch = self.ch;

        if ch == '\0' {
            return Token::Eof;
        }

        match ch {
            '(' => {
                self.read_char();
                let comment = self.read_comment_until_paren();
                return Token::Comment(comment);
            }
            ')' => {
                self.read_char();
                return Token::RParen;
            }
            ';' => {
                self.read_char();
                let comment = self.read_comment_until_newline();
                return Token::Comment(comment);
            }
            '\n' => {
                self.read_char();
                return Token::NewLine;
            }
            _ => {}
        }

        if ch.is_ascii_alphabetic() {
            let letter = ch.to_string();
            self.read_char();

            if letter == "G" || letter == "g" {
                let num = self.read_number();
                return Token::GCode(num as i32);
            }

            if letter == "M" || letter == "m" {
                let num = self.read_number();
                return Token::MCode(num as i32);
            }

            if letter == "N" || letter == "n" {
                let num = self.read_number();
                return Token::Number(num);
            }

            if "XYZABCUVWRFSIJKR".contains(letter.as_str()) {
                let value = self.read_number();
                return Token::Axis(letter, value);
            }

            return Token::Unknown(ch);
        }

        if ch.is_ascii_digit() || ch == '.' {
            let num = self.read_number();
            return Token::Number(num);
        }

        self.read_char();
        Token::Unknown(ch)
    }

    /// Читает число (целое или с плавающей точкой)
    fn read_number(&mut self) -> f64 {
        let mut result = String::new();

        if self.ch == '-' {
            result.push(self.ch);
            self.read_char();
        }

        while self.ch.is_ascii_digit() {
            result.push(self.ch);
            self.read_char();
        }

        if self.ch == '.' {
            result.push(self.ch);
            self.read_char();

            while self.ch.is_ascii_digit() {
                result.push(self.ch);
                self.read_char();
            }
        }

        result.parse::<f64>().unwrap_or(0.0)
    }

    /// Читает комментарий до закрывающей скобки с учётом вложенности
    fn read_comment_until_paren(&mut self) -> String {
        let mut result = String::new();
        let mut paren_depth = 1;

        while self.ch != '\0' {
            if self.ch == '(' {
                paren_depth += 1;
            } else if self.ch == ')' {
                paren_depth -= 1;
                if paren_depth == 0 {
                    self.read_char();
                    return result;
                }
            }
            result.push(self.ch);
            self.read_char();
        }

        result
    }

    /// Читает комментарий до конца строки (после `;`)
    fn read_comment_until_newline(&mut self) -> String {
        let mut result = String::new();

        while self.ch != '\n' && self.ch != '\0' {
            result.push(self.ch);
            self.read_char();
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_token() {
        let input = "G0 X10 Y20 (Rapid move)\nG1 Z5.5 F100";
        let mut lexer = Lexer::new(input.to_string());

        assert_eq!(lexer.next_token(), Token::GCode(0));
        assert_eq!(lexer.next_token(), Token::Axis("X".to_string(), 10.0));
        assert_eq!(lexer.next_token(), Token::Axis("Y".to_string(), 20.0));
        assert_eq!(lexer.next_token(), Token::Comment("Rapid move".to_string()));
        assert_eq!(lexer.next_token(), Token::NewLine);
        assert_eq!(lexer.next_token(), Token::GCode(1));
        assert_eq!(lexer.next_token(), Token::Axis("Z".to_string(), 5.5));
        assert_eq!(lexer.next_token(), Token::Axis("F".to_string(), 100.0));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_semicolon_comment() {
        let input = "G0 X10 ; this is a comment\nG1 Y20";
        let mut lexer = Lexer::new(input.to_string());

        assert_eq!(lexer.next_token(), Token::GCode(0));
        assert_eq!(lexer.next_token(), Token::Axis("X".to_string(), 10.0));
        assert_eq!(
            lexer.next_token(),
            Token::Comment(" this is a comment".to_string())
        );
        assert_eq!(lexer.next_token(), Token::NewLine);
        assert_eq!(lexer.next_token(), Token::GCode(1));
        assert_eq!(lexer.next_token(), Token::Axis("Y".to_string(), 20.0));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_negative_numbers() {
        let input = "G0 X-10 Y-20.5";
        let mut lexer = Lexer::new(input.to_string());

        assert_eq!(lexer.next_token(), Token::GCode(0));
        assert_eq!(lexer.next_token(), Token::Axis("X".to_string(), -10.0));
        assert_eq!(lexer.next_token(), Token::Axis("Y".to_string(), -20.5));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_mcode() {
        let input = "M3 M05 M17";
        let mut lexer = Lexer::new(input.to_string());

        assert_eq!(lexer.next_token(), Token::MCode(3));
        assert_eq!(lexer.next_token(), Token::MCode(5));
        assert_eq!(lexer.next_token(), Token::MCode(17));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_comment_with_parens_inside() {
        let input = "G0 (comment (nested) content) X10";
        let mut lexer = Lexer::new(input.to_string());

        assert_eq!(lexer.next_token(), Token::GCode(0));
        assert_eq!(
            lexer.next_token(),
            Token::Comment("comment (nested) content".to_string())
        );
        assert_eq!(lexer.next_token(), Token::Axis("X".to_string(), 10.0));
        assert_eq!(lexer.next_token(), Token::Eof);
    }
}
