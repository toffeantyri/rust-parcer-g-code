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
mod tests {
    use super::*;

    #[test]
    fn test_simple_gcode() {
        let tokens = tokenize("G0 X10 Y20 (Rapid move)\nG1 Z5.5 F100");

        assert_eq!(tokens[0], Token::GCode(0));
        assert_eq!(tokens[1], Token::Axis("X".to_string(), Some(10.0)));
        assert_eq!(tokens[2], Token::Axis("Y".to_string(), Some(20.0)));
        // Скобки — не комментарий, а скобочное выражение (Word)
        assert_eq!(tokens[3], Token::Word("(Rapid move)".to_string()));
        assert_eq!(tokens[4], Token::NewLine);
        assert_eq!(tokens[5], Token::GCode(1));
        assert_eq!(tokens[6], Token::Axis("Z".to_string(), Some(5.5)));
        assert_eq!(tokens[7], Token::Axis("F".to_string(), Some(100.0)));
    }

    #[test]
    fn test_semicolon_comment() {
        let tokens = tokenize("G0 X10 ; this is a comment\nG1 Y20");

        assert_eq!(tokens[0], Token::GCode(0));
        assert_eq!(tokens[1], Token::Axis("X".to_string(), Some(10.0)));
        assert_eq!(tokens[2], Token::Comment(" this is a comment".to_string()));
        assert_eq!(tokens[3], Token::NewLine);
        assert_eq!(tokens[4], Token::GCode(1));
        assert_eq!(tokens[5], Token::Axis("Y".to_string(), Some(20.0)));
    }

    #[test]
    fn test_negative_numbers() {
        let tokens = tokenize("G0 X-10 Y-20.5");

        assert_eq!(tokens[0], Token::GCode(0));
        assert_eq!(tokens[1], Token::Axis("X".to_string(), Some(-10.0)));
        assert_eq!(tokens[2], Token::Axis("Y".to_string(), Some(-20.5)));
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
        assert_eq!(tokens[2], Token::Axis("X".to_string(), Some(10.0)));
    }

    #[test]
    fn test_multichar_words() {
        let tokens = tokenize("MODECHECK(2) TRANS Z-8 MATLCH(\"DISKD125\",0,1)");

        // Скобочные аргументы — часть слова
        assert_eq!(tokens[0], Token::Word("MODECHECK(2)".to_string()));
        assert_eq!(tokens[1], Token::Word("TRANS".to_string()));
        assert_eq!(tokens[2], Token::Axis("Z".to_string(), Some(-8.0)));
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

        assert_eq!(tokens[0], Token::Axis("Z".to_string(), Some(71.304)));
        assert_eq!(tokens[1], Token::NewLine);
        assert_eq!(tokens[2], Token::Axis("Y".to_string(), Some(-58.346)));
    }

    // -----------------------------------------------------------------------
    // Пограничные случаи
    // -----------------------------------------------------------------------

    #[test]
    fn test_word_with_space_before_paren() {
        // Пробел между многосимвольным словом и скобкой
        let tokens = tokenize("MODECHECK (2)");
        assert_eq!(tokens[0], Token::Word("MODECHECK(2)".to_string()));
    }

    #[test]
    fn test_comment_at_end_of_line() {
        // Комментарий после команды на той же строке
        let tokens = tokenize("G0 X10 ;это комментарий\nG1 Y20");

        assert_eq!(tokens[0], Token::GCode(0));
        assert_eq!(tokens[1], Token::Axis("X".to_string(), Some(10.0)));
        assert_eq!(tokens[2], Token::Comment("это комментарий".to_string()));
        assert_eq!(tokens[3], Token::NewLine);
        assert_eq!(tokens[4], Token::GCode(1));
        assert_eq!(tokens[5], Token::Axis("Y".to_string(), Some(20.0)));
    }

    #[test]
    fn test_unknown_symbols() {
        // Неизвестные символы не должны вызывать панику
        let tokens = tokenize("@#%");
        assert_eq!(tokens[0], Token::Unknown('@'));
        assert_eq!(tokens[1], Token::Unknown('#'));
        assert_eq!(tokens[2], Token::Unknown('%'));
    }

    #[test]
    fn test_axis_with_negative_value() {
        // Ось с отрицательным числом без пробела, без G-кода
        let tokens = tokenize("X-10");
        assert_eq!(tokens[0], Token::Axis("X".to_string(), Some(-10.0)));
    }

    #[test]
    fn test_empty_input() {
        // Пустой ввод не должен паниковать
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        // Только пробелы без перевода строки
        let tokens = tokenize("   \t  ");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_whitespace_with_newlines() {
        // Пробелы с переводами строк — NewLine сохраняются
        let tokens = tokenize("  \n  \n");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::NewLine);
        assert_eq!(tokens[1], Token::NewLine);
    }

    #[test]
    fn test_system_variable() {
        let tokens = tokenize("R50=$TC_MPP6[9998,1]");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Word("R50=$TC_MPP6[9998,1]".to_string()));
    }

    #[test]
    fn test_r_parameter_assign() {
        let tokens = tokenize("R101=R101+1");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Word("R101=R101+1".to_string()));
    }

    #[test]
    fn test_r_parameter_assign_with_spaces() {
        let tokens = tokenize("R100 = 5");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Word("R100=5".to_string()));
    }

    #[test]
    fn test_r_parameter_simple() {
        let tokens = tokenize("G0 R50 X100");
        assert_eq!(tokens[0], Token::GCode(0));
        assert_eq!(tokens[1], Token::Word("R50".to_string()));
        assert_eq!(tokens[2], Token::Axis("X".to_string(), Some(100.0)));
    }

    #[test]
    fn test_r_check() {
        let tokens = tokenize("RCHECK");
        assert_eq!(tokens[0], Token::Word("RCHECK".to_string()));
    }

    #[test]
    fn test_while_condition() {
        let tokens = tokenize("WHILE R101<R103");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Word("WHILE R101<R103".to_string()));
    }

    #[test]
    fn test_while_condition_with_parens() {
        let tokens = tokenize("WHILE (R101<3) AND (R102>0)");
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens[0],
            Token::Word("WHILE (R101<3) AND (R102>0)".to_string())
        );
    }

    #[test]
    fn test_if_condition() {
        let tokens = tokenize("IF R101==0");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Word("IF R101==0".to_string()));
    }

    #[test]
    fn test_endwhile_no_condition() {
        let tokens = tokenize("ENDWHILE");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Word("ENDWHILE".to_string()));
    }

    #[test]
    fn test_endif_no_condition() {
        let tokens = tokenize("ENDIF");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Word("ENDIF".to_string()));
    }

    #[test]
    fn test_else_standalone() {
        let tokens = tokenize("ELSE");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Word("ELSE".to_string()));
    }

    #[test]
    fn test_repeat_condition() {
        let tokens = tokenize("REPEAT R101<R103");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Word("REPEAT R101<R103".to_string()));
    }

    #[test]
    fn test_until_condition() {
        let tokens = tokenize("UNTIL R101>=R103");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Word("UNTIL R101>=R103".to_string()));
    }

    #[test]
    fn test_system_variable_standalone() {
        // Системная переменная без R-параметра
        let tokens = tokenize("$TC_MPP6[9998,1]");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Word("$TC_MPP6[9998,1]".to_string()));
    }

    #[test]
    fn test_while_with_trailing_comment() {
        // WHILE с комментарием — условие не должно включать комментарий
        let tokens = tokenize("WHILE R101<R103 ; loop condition");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Word("WHILE R101<R103".to_string()));
        assert_eq!(tokens[1], Token::Comment(" loop condition".to_string()));
    }

    #[test]
    fn test_if_lowercase() {
        // Ключевые слова регистронезависимы
        let tokens = tokenize("if R101==0");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Word("if R101==0".to_string()));
    }

    #[test]
    fn test_until_with_spaces() {
        // UNTIL с пробелами вокруг скобок
        let tokens = tokenize("UNTIL (R101>=R103)");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Word("UNTIL (R101>=R103)".to_string()));
    }

    #[test]
    fn test_if_with_parens() {
        let tokens = tokenize("IF (R101==0) AND (R102>5)");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Word("IF (R101==0) AND (R102>5)".to_string()));
    }
}
