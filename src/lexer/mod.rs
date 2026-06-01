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
    /// # Пример
    /// ```
    /// let lexer = Lexer::new("G0 X10 Y20".to_string());
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
        // Здесь будет логика определения токенов
        todo!("Implement tokenization logic")
    }

    /// Пропускает все пробельные символы в начале токена
    /// 
    /// Метод анализирует текущий символ и, если он является пробельным (пробел, табуляция и т.д.),
    /// последовательно считывает следующие символы до тех пор, пока не встретится непробельный символ.
    /// Это позволяет игнорировать пробелы между токенами в программе G-кода.
    fn skip_whitespace(&mut self) {
        while self.ch.is_whitespace() {
            self.read_char();
        }
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
}