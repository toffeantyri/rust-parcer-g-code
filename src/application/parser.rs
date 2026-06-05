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
            Token::Number(value) => value.to_string(),
            Token::Comment(text) => format!("({})", text),
            Token::LParen => "(".to_string(),
            Token::RParen => ")".to_string(),
            Token::Semicolon => ";".to_string(),
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

    fn make_tokens(input: &str) -> Vec<Token> {
        let mut lexer = crate::infrastructure::Lexer::new(input.to_string());
        let mut tokens = Vec::new();
        loop {
            let tok = lexer.next_token();
            if tok == Token::Eof {
                break;
            }
            tokens.push(tok);
        }
        tokens
    }

    #[test]
    fn test_parse_simple_program() {
        let tokens = make_tokens("G0 X10 Y20\nG1 Z5.5");
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
}
