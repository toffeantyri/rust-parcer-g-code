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
            '$' => {
                return self.read_system_variable();
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
                    if "XYZABCUVWFSIJK".contains(letter.as_str()) {
                        // Если после буквы идёт `=`, читаем выражение
                        if self.ch == '=' {
                            return self.read_axis_expr(letter);
                        }
                        // Если после буквы идёт число, минус или точка — читаем числовое значение оси
                        if self.ch.is_ascii_digit() || self.ch == '-' || self.ch == '.' {
                            let value = self.read_number();
                            return Token::Axis(letter, Some(value));
                        }
                        // Ось без числа — оставляем None (будет ошибкой валидации)
                        return Token::Axis(letter, None);
                    }
                    // Проверяем R-параметр перед тем как вернуть как Word
                    if letter.as_str() == "R" && (self.ch.is_ascii_digit() || self.ch == '=') {
                        return self.read_r_parameter(letter);
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

        // Проверяем, является ли слово ключевым словом управления потоком
        // (до того как word будет перемещён в full_word)
        let is_flow_control = {
            let upper = word.to_uppercase();
            upper == "WHILE" || upper == "IF" || upper == "ELSE" || upper == "REPEAT" || upper == "UNTIL"
        };

        let mut full_word = word;

        // Для ключевых слов управления потоком захватываем всё условие до конца
        // строки целиком (включая скобки), не разбирая скобки отдельно.
        if is_flow_control {
            // Пропускаем пробелы перед условием
            while self.ch.is_whitespace() && self.ch != '\n' {
                self.read_char();
            }
            // Читаем всё до конца строки или ';'
            if self.ch != '\n' && self.ch != ';' && self.ch != '\0' {
                full_word.push(' ');
                while self.ch != '\n' && self.ch != ';' && self.ch != '\0' {
                    full_word.push(self.ch);
                    self.read_char();
                }
                // Убираем trailing пробелы
                while full_word.ends_with(' ') {
                    full_word.pop();
                }
            }
        } else {
            // Если после слова (возможно с пробелом) идёт `(`, читаем весь блок в скобках
            if self.ch == '(' {
                full_word.push('(');
                self.read_char();
                let args = self.read_parenthesized_content();
                full_word.push_str(&args);
                full_word.push(')');
            }
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

    /// Читает системную переменную Siemens: $TC_MPP6[9998,1] и т.д.
    /// Начинается с '$', читает всё до пробела / новой строки / ';'.
    fn read_system_variable(&mut self) -> Token {
        let mut var = String::from("$");
        self.read_char(); // пропускаем $
        while self.ch != '\0' && !self.ch.is_whitespace() && self.ch != ';' && self.ch != '\n' {
            var.push(self.ch);
            self.read_char();
        }
        Token::Word(var)
    }

    /// Читает R-параметр (R50, R101...) и последующее присваивание если есть.
    /// Возвращает единый Word: "R50", "R101=R101+1" и т.д.
    fn read_r_parameter(&mut self, prefix: String) -> Token {
        let mut full = prefix;
        // Читаем число после R (например R50, R101)
        if self.ch.is_ascii_digit() {
            while self.ch.is_ascii_digit() {
                full.push(self.ch);
                self.read_char();
            }
        }
        // Пропускаем пробелы перед '=' (бывает "R100 = ...")
        while self.ch.is_whitespace() && self.ch != '\n' {
            self.read_char();
        }
        // Если после числа/пробелов идёт '=', читаем всё выражение присваивания
        if self.ch == '=' {
            full.push('=');
            self.read_char(); // пропускаем =
            // Пропускаем пробелы после '=' (бывает "R100 = 5")
            while self.ch.is_whitespace() && self.ch != '\n' {
                self.read_char();
            }
            // Читаем всё до пробела, новой строки или ';'
            while self.ch != '\0' && !self.ch.is_whitespace() && self.ch != ';' && self.ch != '\n' {
                full.push(self.ch);
                self.read_char();
            }
        }
        Token::Word(full)
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
#[path = "lexer_tests.rs"]
mod tests;
