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
            self.position = self.input.len();
            self.read_position = self.input.len() + 1;
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

        // Если символ — буква, читаем всё слово целиком
        if ch.is_ascii_alphabetic() {
            return self.read_word();
        }

        // Если символ — открывающая скобка, читаем как скобочное выражение
        if ch == '(' {
            return self.read_paren_expr();
        }

        // Закрывающая скобка отдельно — Unknown (не должна встречаться)
        if ch == ')' {
            self.read_char();
            return Token::Unknown(')');
        }

        // Цифры и десятичная точка — число
        if ch.is_ascii_digit() || ch == '.' {
            let num = self.read_number();
            return Token::Number(num);
        }

        self.read_char();
        Token::Unknown(ch)
    }

    /// Читает буквенное слово и определяет его тип
    ///
    /// Если слово состоит из одной буквы и за ним идёт число — это G-код, M-код, ось или N-код.
    /// Иначе — многосимвольное слово (Word). Для многосимвольных слов захватываем
    /// также аргументы в скобках как часть слова (например MODECHECK(2)).
    fn read_word(&mut self) -> Token {
        let start = self.position;
        // Читаем все буквы подряд
        while self.ch.is_ascii_alphabetic() {
            self.read_char();
        }
        let word: String = self.input[start..self.position].to_string();

        // Проверяем односимвольные коды: G, M, N, оси
        if word.len() == 1 {
            let letter = word.to_uppercase();
            match letter.as_str() {
                "G" => {
                    let num = self.read_number();
                    return Token::GCode(num as i32);
                }
                "M" => {
                    let num = self.read_number();
                    return Token::MCode(num as i32);
                }
                "N" => {
                    let num = self.read_number();
                    return Token::NCode(num as i32);
                }
                _ => {
                    // Проверяем, является ли буква осью
                    if "XYZABCUVWRFSIJKR".contains(letter.as_str()) {
                        // Если после буквы идёт `=`, читаем выражение
                        if self.ch == '=' {
                            return self.read_axis_expr(letter);
                        }
                        // Если после буквы идёт число или минус — читаем числовое значение оси
                        if self.ch.is_ascii_digit() || self.ch == '-' || self.ch == '.' {
                            let value = self.read_number();
                            return Token::Axis(letter, value);
                        }
                    }
                    // Одиночная неизвестная буква — Word
                    return Token::Word(word);
                }
            }
        }

        // Многосимвольное слово — захватываем скобочные аргументы как часть слова
        // Пропускаем пробелы перед скобкой (бывает "MODECHECK (2)")
        let mut after_word = self.ch;
        while after_word.is_whitespace() && after_word != '\n' {
            self.read_char();
            after_word = self.ch;
        }

        let mut full_word = word;
        // Если после слова (возможно с пробелом) идёт `(`, читаем весь блок в скобках
        if self.ch == '(' {
            full_word.push('(');
            self.read_char();
            let args = self.read_parenthesized_content();
            full_word.push_str(&args);
            full_word.push(')');
        }

        Token::Word(full_word)
    }

    /// Читает содержимое внутри скобок до парной `)` с учётом вложенности.
    /// Не пропускает начальную `(` — она уже должна быть считана.
    fn read_parenthesized_content(&mut self) -> String {
        let mut result = String::new();
        let mut depth = 1;

        while self.ch != '\0' {
            if self.ch == '(' {
                depth += 1;
                result.push(self.ch);
                self.read_char();
            } else if self.ch == ')' {
                depth -= 1;
                if depth == 0 {
                    self.read_char(); // пропускаем `)`
                    return result;
                }
                result.push(self.ch);
                self.read_char();
            } else {
                result.push(self.ch);
                self.read_char();
            }
        }

        result
    }

    /// Читает скобочное выражение, которое начинается с `(`.
    /// Возвращает Word со всем содержимым внутри скобок.
    /// Используется когда `(` встречается сама по себе (не после слова).
    fn read_paren_expr(&mut self) -> Token {
        let mut content = String::from("(");
        self.read_char(); // пропускаем `(`
        let inner = self.read_parenthesized_content();
        content.push_str(&inner);
        content.push(')');
        Token::Word(content)
    }

    /// Читает выражение оси после `=`: ось + `=` + арифметическое выражение до пробела.
    /// Например Z=71.304, X=160+10, Y=100*2.
    fn read_axis_expr(&mut self, axis: String) -> Token {
        self.read_char(); // пропускаем `=`
        let mut expr = String::new();
        // Читаем всё до пробела, новой строки или `;`
        while self.ch != '\0' && !self.ch.is_whitespace() && self.ch != ';' {
            expr.push(self.ch);
            self.read_char();
        }
        Token::AxisExpr(axis, expr)
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

/// Вспомогательная функция: собирает все токены из строки в вектор
pub fn tokenize(input: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(input.to_string());
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        if token == Token::Eof {
            break;
        }
        tokens.push(token);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_gcode() {
        let tokens = tokenize("G0 X10 Y20 (Rapid move)\nG1 Z5.5 F100");

        assert_eq!(tokens[0], Token::GCode(0));
        assert_eq!(tokens[1], Token::Axis("X".to_string(), 10.0));
        assert_eq!(tokens[2], Token::Axis("Y".to_string(), 20.0));
        // Скобки — не комментарий, а скобочное выражение (Word)
        assert_eq!(tokens[3], Token::Word("(Rapid move)".to_string()));
        assert_eq!(tokens[4], Token::NewLine);
        assert_eq!(tokens[5], Token::GCode(1));
        assert_eq!(tokens[6], Token::Axis("Z".to_string(), 5.5));
        assert_eq!(tokens[7], Token::Axis("F".to_string(), 100.0));
    }

    #[test]
    fn test_semicolon_comment() {
        let tokens = tokenize("G0 X10 ; this is a comment\nG1 Y20");

        assert_eq!(tokens[0], Token::GCode(0));
        assert_eq!(tokens[1], Token::Axis("X".to_string(), 10.0));
        assert_eq!(tokens[2], Token::Comment(" this is a comment".to_string()));
        assert_eq!(tokens[3], Token::NewLine);
        assert_eq!(tokens[4], Token::GCode(1));
        assert_eq!(tokens[5], Token::Axis("Y".to_string(), 20.0));
    }

    #[test]
    fn test_negative_numbers() {
        let tokens = tokenize("G0 X-10 Y-20.5");

        assert_eq!(tokens[0], Token::GCode(0));
        assert_eq!(tokens[1], Token::Axis("X".to_string(), -10.0));
        assert_eq!(tokens[2], Token::Axis("Y".to_string(), -20.5));
    }

    #[test]
    fn test_axis_expr() {
        let tokens = tokenize("Z=71.304 X=160+10 Y=3*5/2");

        assert_eq!(
            tokens[0],
            Token::AxisExpr("Z".to_string(), "71.304".to_string())
        );
        assert_eq!(
            tokens[1],
            Token::AxisExpr("X".to_string(), "160+10".to_string())
        );
        assert_eq!(
            tokens[2],
            Token::AxisExpr("Y".to_string(), "3*5/2".to_string())
        );
    }

    #[test]
    fn test_mcode() {
        let tokens = tokenize("M3 M05 M17");

        assert_eq!(tokens[0], Token::MCode(3));
        assert_eq!(tokens[1], Token::MCode(5));
        assert_eq!(tokens[2], Token::MCode(17));
    }

    #[test]
    fn test_paren_expr() {
        // Скобки — не комментарий, а часть команды (словесной или параметров)
        let tokens = tokenize("G0 (comment (nested) content) X10");

        assert_eq!(tokens[0], Token::GCode(0));
        // Всё в скобках — Word (скобочное выражение)
        assert_eq!(
            tokens[1],
            Token::Word("(comment (nested) content)".to_string())
        );
        assert_eq!(tokens[2], Token::Axis("X".to_string(), 10.0));
    }

    #[test]
    fn test_multichar_words() {
        let tokens = tokenize("MODECHECK(2) TRANS Z-8 MATLCH(\"DISKD125\",0,1)");

        // Скобочные аргументы — часть слова
        assert_eq!(tokens[0], Token::Word("MODECHECK(2)".to_string()));
        assert_eq!(tokens[1], Token::Word("TRANS".to_string()));
        assert_eq!(tokens[2], Token::Axis("Z".to_string(), -8.0));
        assert_eq!(
            tokens[3],
            Token::Word("MATLCH(\"DISKD125\",0,1)".to_string())
        );
    }

    #[test]
    fn test_n_codes() {
        let tokens = tokenize("N0100 G64 CFTCP");

        // N-номер сохраняется как NCode с целым числом
        assert_eq!(tokens[0], Token::NCode(100));
        assert_eq!(tokens[1], Token::GCode(64));
        assert_eq!(tokens[2], Token::Word("CFTCP".to_string()));
    }

    #[test]
    fn test_axis_without_gcode() {
        // Строки начинающиеся с оси (продолжение предыдущего G-кода)
        let tokens = tokenize(" Z71.304\n Y-58.346");

        assert_eq!(tokens[0], Token::Axis("Z".to_string(), 71.304));
        assert_eq!(tokens[1], Token::NewLine);
        assert_eq!(tokens[2], Token::Axis("Y".to_string(), -58.346));
    }
}
