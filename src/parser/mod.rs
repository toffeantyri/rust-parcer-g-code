// Парсер G-кода

use crate::ast::*;
use crate::lexer::{Lexer, Token};

/// Основная структура парсера G-кода
/// 
/// Парсер отвечает за преобразование последовательности токенов, полученных от лексера,
/// в абстрактное синтаксическое дерево (AST), которое представляет собой структурированное
/// представление программы G-кода. Эта структура хранит текущее состояние разбора.
pub struct Parser {
    /// Лексер, предоставляющий токены для разбора
    lexer: Lexer,
    /// Текущий обрабатываемый токен
    current_token: Token,
}

impl Parser {
    /// Создает новый экземпляр парсера для разбора последовательности токенов
    /// 
    /// # Аргументы
    /// * `lexer` - экземпляр лексера, который будет предоставлять токены для разбора
    /// 
    /// # Возвращает
    /// Новый экземпляр Parser, готовый к разбору программы
    /// 
    /// # Пример
    /// ```
    /// let lexer = Lexer::new("G0 X10".to_string());
    /// let mut parser = Parser::new(lexer);
    /// ```
    pub fn new(mut lexer: Lexer) -> Self {
        let current_token = lexer.next_token();
        Parser { lexer, current_token }
    }

    /// Разбирает всю программу G-кода в абстрактное синтаксическое дерево
    /// 
    /// Метод последовательно обрабатывает токены от лексера до тех пор, пока не достигнет
    /// конца входного потока (Eof). Каждый токен преобразуется в соответствующий оператор
    /// в AST, который добавляется в результирующий вектор.
    /// 
    /// # Возвращает
    /// Вектор операторов, представляющих структурированное представление программы G-кода
    /// 
    /// # Пример
    /// ```
    /// let lexer = Lexer::new("G0 X10\nG1 Z5.5".to_string());
    /// let mut parser = Parser::new(lexer);
    /// let program = parser.parse_program();
    /// assert_eq!(program.len(), 5);
    /// ```
    pub fn parse_program(&mut self) -> Vec<Statement> {
        let mut program = Vec::new();

        while !self.current_token_is(&Token::Eof) {
            if let Some(stmt) = self.parse_statement() {
                program.push(stmt);
            }
            self.next_token();
        }

        program
    }

    /// Разбирает текущий токен в соответствующий оператор программы
    /// 
    /// Метод анализирует текущий токен и создает соответствующую структуру оператора
    /// в абстрактном синтаксическом дереве. Для неизвестных токенов создается Raw-оператор,
    /// чтобы сохранить оригинальный формат специфических конструкций без их изменения.
    /// 
    /// # Возвращает
    /// Некоторый оператор (Some(Statement)) если токен может быть распознан,
    /// или None если токен не может быть обработан (что маловероятно при корректной работе лексера)
    fn parse_statement(&mut self) -> Option<Statement> {
        match &self.current_token {
            Token::GCode(code) => Some(Statement::Motion(MotionStatement {
                code: *code,
                rapid: *code == 0,
            })),
            Token::MCode(code) => Some(Statement::Misc(MiscStatement { code: *code })),
            Token::Axis(axis, value) => Some(Statement::Axis(AxisStatement {
                axis: axis.clone(),
                value: *value,
            })),
            Token::Comment(text) => Some(Statement::Comment(CommentStatement { text: text.clone() })),
            // Для любых неизвестных токенов или специфических конструкций
            _ => {
                // Сохраняем оригинальное представление, чтобы не терять специфические конструкции
                let raw = self.current_token_to_string();
                Some(Statement::Raw(raw))
            }
        }
    }

    fn next_token(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    fn current_token_is(&self, token: &Token) -> bool {
        self.current_token == *token
    }

    fn current_token_to_string(&self) -> String {
        // Преобразуем текущий токен в строку для сохранения оригинального представления
        match &self.current_token {
            Token::GCode(code) => format!("G{}", code),
            Token::MCode(code) => format!("M{}", code),
            Token::Axis(axis, value) => format!("{}{}", axis, value),
            Token::Number(value) => value.to_string(),
            Token::Comment(text) => format!("({})", text),
            Token::LParen => "(".to_string(),
            Token::RParen => ")".to_string(),
            Token::Semicolon => ";".to_string(),
            Token::NewLine => "\n".to_string(),
            Token::Unknown(ch) => ch.to_string(),
            Token::Eof => "".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_program() {
        let input = "G0 X10 Y20\nG1 Z5.5 F100";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        assert_eq!(program.len(), 6);
        assert_eq!(program[0], Statement::Motion(MotionStatement { code: 0, rapid: true }));
        assert_eq!(program[1], Statement::Axis(AxisStatement { axis: "X".to_string(), value: 10.0 }));
        assert_eq!(program[2], Statement::Axis(AxisStatement { axis: "Y".to_string(), value: 20.0 }));
        assert_eq!(program[3], Statement::NewLine);
        assert_eq!(program[4], Statement::Motion(MotionStatement { code: 1, rapid: false }));
        assert_eq!(program[5], Statement::Axis(AxisStatement { axis: "Z".to_string(), value: 5.5 }));
    }
}