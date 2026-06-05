//! Прикладной слой: парсер — преобразует поток токенов в AST
//!
//! Использует доменные типы Statement и Token. Ошибки возвращает через ParseError.

use crate::domain::{
    AxisStatement, CommentStatement, MiscStatement, MotionStatement, Statement, Token,
};
use crate::shared::ParseError;

/// Парсер преобразует последовательность токенов в абстрактное синтаксическое дерево
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    /// Создаёт парсер из готового вектора токенов
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    /// Разбирает все токены в программу (вектор операторов)
    pub fn parse_program(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut program = Vec::new();

        while self.position < self.tokens.len() {
            let stmt = self.parse_statement()?;
            if let Some(statement) = stmt {
                program.push(statement);
            }
            self.advance();
        }

        Ok(program)
    }

    /// Разбирает один токен в оператор
    fn parse_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        let token = self.current_token();
        match token {
            Token::GCode(code) => Ok(Some(Statement::Motion(MotionStatement {
                code: *code,
                rapid: *code == 0,
            }))),
            Token::MCode(code) => Ok(Some(Statement::Misc(MiscStatement { code: *code }))),
            Token::Axis(axis, value) => Ok(Some(Statement::Axis(AxisStatement {
                axis: axis.clone(),
                value: *value,
            }))),
            Token::Comment(text) => Ok(Some(Statement::Comment(CommentStatement {
                text: text.clone(),
            }))),
            Token::NewLine => Ok(Some(Statement::NewLine)),
            Token::NCode(code) => Ok(Some(Statement::NCode(*code))),
            Token::Word(word) => Ok(Some(Statement::Word(word.clone()))),
            Token::Eof => Ok(None),
            _ => {
                let raw = self.token_to_string(token);
                Ok(Some(Statement::Raw(raw)))
            }
        }
    }

    /// Возвращает ссылку на текущий токен
    fn current_token(&self) -> &Token {
        &self.tokens[self.position]
    }

    /// Переходит к следующему токену
    fn advance(&mut self) {
        self.position += 1;
    }

    /// Преобразует токен в строку для Raw-оператора
    fn token_to_string(&self, token: &Token) -> String {
        match token {
            Token::GCode(code) => format!("G{}", code),
            Token::MCode(code) => format!("M{}", code),
            Token::Axis(axis, value) => format!("{}{}", axis, value),
            Token::Word(word) => word.clone(),
            Token::NCode(code) => format!("N{:04}", code),
            Token::Number(value) => value.to_string(),
            Token::Comment(text) => format!(";{}", text),
            Token::NewLine => "\n".to_string(),
            Token::Unknown(ch) => ch.to_string(),
            Token::Eof => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::*;
    use crate::infrastructure::lexer::tokenize;

    #[test]
    fn test_parse_simple_program() {
        let tokens = tokenize("G0 X10 Y20\nG1 Z5.5");
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.len(), 6);
        assert_eq!(
            program[0],
            Statement::Motion(MotionStatement {
                code: 0,
                rapid: true
            })
        );
        assert_eq!(
            program[1],
            Statement::Axis(AxisStatement {
                axis: "X".to_string(),
                value: 10.0
            })
        );
        assert_eq!(
            program[2],
            Statement::Axis(AxisStatement {
                axis: "Y".to_string(),
                value: 20.0
            })
        );
        assert_eq!(program[3], Statement::NewLine);
        assert_eq!(
            program[4],
            Statement::Motion(MotionStatement {
                code: 1,
                rapid: false
            })
        );
        assert_eq!(
            program[5],
            Statement::Axis(AxisStatement {
                axis: "Z".to_string(),
                value: 5.5
            })
        );
    }

    #[test]
    fn test_parse_multichar_words() {
        // Многосимвольные команды со скобками — единый Raw
        let tokens = tokenize("G64 CFTCP\nMODECHECK(2)");
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();

        // G64, CFTCP, NewLine, MODECHECK(2)
        assert_eq!(program.len(), 4);
        assert_eq!(
            program[0],
            Statement::Motion(MotionStatement {
                code: 64,
                rapid: false
            })
        );
        assert_eq!(program[1], Statement::Word("CFTCP".to_string()));
        assert_eq!(program[2], Statement::NewLine);
        assert_eq!(program[3], Statement::Word("MODECHECK(2)".to_string()));
    }

    #[test]
    fn test_parse_n_codes() {
        // N-номер становится Number-токеном, который парсер превращает в Raw
        let tokens = tokenize("N0100 G0");
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.len(), 2);
        assert_eq!(program[0], Statement::NCode(100));
        assert_eq!(
            program[1],
            Statement::Motion(MotionStatement {
                code: 0,
                rapid: true
            })
        );
    }

    #[test]
    fn test_parse_full_input_snapshot() {
        // Проверяем, что весь файл input_code.txt парсится без ошибок
        let input = std::fs::read_to_string("input_code.txt")
            .expect("input_code.txt должен существовать в корне проекта");
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();

        // Программа не должна быть пустой
        assert!(!program.is_empty(), "Программа должна содержать операторы");

        // Проверяем, что есть ключевые типы операторов
        let has_motion = program.iter().any(|s| matches!(s, Statement::Motion(_)));
        let has_axis = program.iter().any(|s| matches!(s, Statement::Axis(_)));
        let has_ncode = program.iter().any(|s| matches!(s, Statement::NCode(_)));
        let has_word = program.iter().any(|s| matches!(s, Statement::Word(_)));
        let has_newline = program.iter().any(|s| *s == Statement::NewLine);

        assert!(has_motion, "Должен быть хотя бы один MotionStatement");
        assert!(has_axis, "Должен быть хотя бы один AxisStatement");
        assert!(has_ncode, "Должны быть N-коды");
        assert!(has_word, "Должны быть словесные команды (Word)");
        assert!(has_newline, "Должны быть NewLine");
    }
}
