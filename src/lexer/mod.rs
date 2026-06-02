// Лексер для разбора G-кода

/// Основной структура лексера G-кода
/// 
/// Лексер отвечает за преобразование входного текста программы в последовательность токенов,
/// которые затем используются парсером для построения абстрактного синтаксического дерева.
/// Эта структура хранит текущее состояние разбора: позицию в исходном тексте и текущий символ.
pub struct Lexer {
    /// Исходный текст программы G-кода, который необходимо разобрать
    input: String,
    /// Текущая позиция чтения в строке (индекс символа)
    position: usize,
    /// Позиция следующего символа для чтения
    read_position: usize,
    /// Текущий обрабатываемый символ
    ch: char,
}

impl Lexer {
    /// Создает новый экземпляр лексера для разбора текста программы G-кода
    /// 
    /// # Аргументы
    /// * `input` - строка с текстом программы G-кода, который необходимо разобрать
    /// 
    /// # Возвращает
    /// Новый экземпляр Lexer, готовый к токенизации
    /// 
"    /// # Пример\n    /// ```\n    /// use code_parser::lexer::{Lexer, Token};\n    /// let lexer = Lexer::new(\"G0 X10 Y20\".to_string());"}
    /// ```
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

    /// Считывает следующий символ из входной строки и обновляет позиции
    /// 
    /// Этот метод перемещает указатель чтения на следующий символ во входной строке
    /// и обновляет текущий символ, который будет использоваться при разборе.
    /// Если достигнут конец строки, устанавливается нулевой символ '\0'.
    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0';
        } else {
            self.ch = self.input.chars().nth(self.read_position).unwrap();
        }
        self.position = self.read_position;
        self.read_position += 1;
    }

    /// Пропускает все пробельные символы в начале токена (кроме новой строки)
    /// 
    /// Метод анализирует текущий символ и, если он является пробельным (пробел, табуляция и т.д.),
    /// последовательно считывает следующие символы до тех пор, пока не встретится непробельный символ.
    /// Это позволяет игнорировать пробелы между токенами в программе G-кода.
    fn skip_whitespace(&mut self) {
        while self.ch.is_whitespace() && self.ch != '\n' {
            self.read_char();
        }
    }

    /// Возвращает следующий токен из входного потока
    /// 
    /// Метод пропускает пробельные символы, затем анализирует текущий символ
    /// и определяет, к какому типу токена он относится (G-код, M-код, ось координат и т.д.).
    /// После определения токена автоматически переходит к следующему символу.
    /// 
    /// # Возвращает
    /// Токен, представляющий следующую логическую единицу в программе G-кода
    /// 
    /// # Пример
    /// ```
    /// let mut lexer = Lexer::new("G0 X10".to_string());
    /// assert_eq!(lexer.next_token(), Token::GCode(0));
    /// assert_eq!(lexer.next_token(), Token::Axis("X".to_string(), 10.0));
    /// ```
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        // Сохраняем начальную позицию для извлечения полного токена
        let ch = self.ch;

        // Проверяем конец файла
        if ch == '\0' {
            return Token::Eof;
        }

        // Обрабатываем односимвольные токены
        match ch {
            '(' => {
                self.read_char(); // Пропускаем (
                let comment = self.read_comment_until_paren();
                return Token::Comment(comment);
            }
            ')' => {
                self.read_char();
                return Token::RParen;
            }
            ';' => {
                // Пропускаем ; и читаем комментарий до конца строки
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

        // Проверяем буквы для G, M, осей и N-кодов
        if ch.is_ascii_alphabetic() {
            // Читаем букву (одиночную)
            let letter = ch.to_string();
            self.read_char();
            
            // Проверяем, является ли это G-кодом
            if letter == "G" || letter == "g" {
                let num = self.read_number();
                return Token::GCode(num as i32);
            }
            
            // Проверяем, является ли это M-кодом
            if letter == "M" || letter == "m" {
                let num = self.read_number();
                return Token::MCode(num as i32);
            }
            
            // Проверяем, является ли это N-кодом (номер программы)
            if letter == "N" || letter == "n" {
                let num = self.read_number();
                return Token::Number(num);
            }
            
            // Проверяем, является ли это осью координаты
            if "XYZABCUVWRFSIJKR".contains(letter.as_str()) {
                // Читаем число после буквы
                let value = self.read_number();
                return Token::Axis(letter, value);
            }
            
            // Для других букв возвращаем как неизвестный токен
            return Token::Unknown(ch);
        }

        // Проверяем цифры и десятичные числа
        if ch.is_ascii_digit() || ch == '.' {
            let num = self.read_number();
            return Token::Number(num);
        }

        // Если символ не распознан
        self.read_char();
        Token::Unknown(ch)
    }

    /// Читает число (целое или с плавающей точкой)
    fn read_number(&mut self) -> f64 {
        let mut result = String::new();
        
        // Обработка знака минус
        if self.ch == '-' {
            result.push(self.ch);
            self.read_char();
        }
        
        // Целая часть
        while self.ch.is_ascii_digit() {
            result.push(self.ch);
            self.read_char();
        }
        
        // Десятичная точка и дробная часть
        if self.ch == '.' {
            result.push(self.ch);
            self.read_char();
            
            while self.ch.is_ascii_digit() {
                result.push(self.ch);
                self.read_char();
            }
        }
        
        // Попытка парсинга числа
        result.parse::<f64>().unwrap_or(0.0)
    }

    /// Читает комментарий до закрывающей скобки с учетом вложенных скобок
    fn read_comment_until_paren(&mut self) -> String {
        let mut result = String::new();
        let mut paren_depth = 1; // Начинаем внутри первой скобки
        
        while self.ch != '\0' {
            if self.ch == '(' {
                paren_depth += 1;
            } else if self.ch == ')' {
                paren_depth -= 1;
                if paren_depth == 0 {
                    self.read_char(); // Пропускаем закрывающую скобку
                    return result;
                }
            }
            result.push(self.ch);
            self.read_char();
        }
        
        result
    }

    /// Читает комментарий до конца строки (для ; комментариев)
    fn read_comment_until_newline(&mut self) -> String {
        let mut result = String::new();
        
        while self.ch != '\n' && self.ch != '\0' {
            result.push(self.ch);
            self.read_char();
        }
        
        result
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Основные токены G-кода
    GCode(i32),        // G0, G1, G2 и т.д.
    MCode(i32),        // M3, M5 и т.д.
    Axis(String, f64), // X10.5, Y20.0 и т.д.
    Number(f64),
    Comment(String),   // Комментарии в скобках или после ;
    Eof,
    // Специальные символы
    LParen,            // (
    RParen,            // )
    Semicolon,         // ;
    NewLine,
    Unknown(char),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_token() {
        let input = "G0 X10 Y20 (Rapid move)\nG1 Z5.5 F100";
        let mut lexer = Lexer::new(input.to_string());

        // Добавьте тесты для проверки токенизации
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
        assert_eq!(lexer.next_token(), Token::Comment(" this is a comment".to_string()));
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
    fn test_parentheses() {
        let input = "(comment) G0";
        let mut lexer = Lexer::new(input.to_string());

        assert_eq!(lexer.next_token(), Token::Comment("comment".to_string()));
        assert_eq!(lexer.next_token(), Token::GCode(0));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_n_code() {
        let input = "N0100 G0";
        let mut lexer = Lexer::new(input.to_string());

        assert_eq!(lexer.next_token(), Token::Number(100.0));
        assert_eq!(lexer.next_token(), Token::GCode(0));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_comment_with_parens_inside() {
        let input = "G0 (comment (nested) content) X10";
        let mut lexer = Lexer::new(input.to_string());

        assert_eq!(lexer.next_token(), Token::GCode(0));
        assert_eq!(lexer.next_token(), Token::Comment("comment (nested) content".to_string()));
        assert_eq!(lexer.next_token(), Token::Axis("X".to_string(), 10.0));
        assert_eq!(lexer.next_token(), Token::Eof);
    }
}
