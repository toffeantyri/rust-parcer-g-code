//! Прикладной слой: парсер — преобразует поток токенов в AST
//!
//! Использует доменные типы Statement и Token. Ошибки возвращает через ParseError.

use crate::domain::{
    AxisStatement, CommentStatement, IfStatement, MiscStatement, MotionStatement, Statement, Token,
    WhileStatement,
};
use crate::shared::ParseError;

/// Парсер преобразует последовательность токенов в абстрактное синтаксическое дерево
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    pub fn parse_program(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut program = Vec::new();

        while self.position < self.tokens.len() {
            // Клонируем токен чтобы избежать двойного заимствования self
            let tok = self.tokens[self.position].clone();
            let stmt = self.parse_token(&tok)?;
            if let Some(statement) = stmt {
                program.push(statement);
            }
            self.advance();
        }

        Ok(program)
    }

    /// Разбирает один токен в оператор (не требует &mut self для чтения)
    fn parse_token(&mut self, token: &Token) -> Result<Option<Statement>, ParseError> {
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
            Token::AxisExpr(axis, expr) => {
                Ok(Some(Statement::Word(format!("{}={}", axis, expr))))
            }
            Token::Comment(text) => Ok(Some(Statement::Comment(CommentStatement {
                text: text.clone(),
            }))),
            Token::NewLine => Ok(Some(Statement::NewLine)),
            Token::NCode(code) => Ok(Some(Statement::NCode(*code))),
            Token::Word(word) => {
                let w = word.clone();
                self.parse_word(&w)
            }
            Token::Eof => Ok(None),
            _ => {
                let raw = self.token_to_string(token);
                Ok(Some(Statement::Raw(raw)))
            }
        }
    }

    fn parse_word(&mut self, word: &str) -> Result<Option<Statement>, ParseError> {
        let upper = word.to_uppercase();

        if upper.starts_with("WHILE") {
            return self.parse_while_block(word);
        }

        if upper == "IF" || upper.starts_with("IF ") {
            return self.parse_if_block(word);
        }

        Ok(Some(Statement::Word(word.to_string())))
    }

    fn parse_while_block(&mut self, word: &str) -> Result<Option<Statement>, ParseError> {
        let condition = if word.len() > 6 {
            word[6..].to_string()
        } else {
            String::new()
        };

        let mut body = Vec::new();
        let mut depth = 1u32;

        while self.position + 1 < self.tokens.len() {
            self.advance();
            let tok = self.tokens[self.position].clone();

            match &tok {
                Token::Word(w) => {
                    let up = w.to_uppercase();
                    if up == "ENDWHILE" {
                        depth -= 1;
                        if depth == 0 {
                            return Ok(Some(Statement::WhileBlock(WhileStatement {
                                condition,
                                body,
                            })));
                        }
                        body.push(Statement::Word(w.clone()));
                    } else if up.starts_with("WHILE") {
                        // Вложенный WHILE — парсим рекурсивно
                        // depth не увеличиваем: рекурсивный вызов сам обработает
                        // свой ENDWHILE, а внешний ENDWHILE закроет внешний цикл.
                        let inner = self.parse_while_block_inline(w)?;
                        body.push(inner);
                    } else if up.starts_with("IF") {
                        let if_stmt = self.parse_if_block_inline(w)?;
                        body.push(if_stmt);
                    } else {
                        body.push(Statement::Word(w.clone()));
                    }
                }
                Token::NewLine => body.push(Statement::NewLine),
                Token::NCode(code) => body.push(Statement::NCode(*code)),
                Token::GCode(code) => body.push(Statement::Motion(MotionStatement {
                    code: *code,
                    rapid: *code == 0,
                })),
                Token::MCode(code) => {
                    body.push(Statement::Misc(MiscStatement { code: *code }))
                }
                Token::Axis(axis, value) => body.push(Statement::Axis(AxisStatement {
                    axis: axis.clone(),
                    value: *value,
                })),
                Token::AxisExpr(axis, expr) => body.push(Statement::Word(format!(
                    "{}={}",
                    axis, expr
                ))),
                Token::Comment(text) => body.push(Statement::Comment(CommentStatement {
                    text: text.clone(),
                })),
                Token::Eof => {
                    return Err(ParseError {
                        message: "WHILE без ENDWHILE".into(),
                        position: self.position,
                    });
                }
                _ => {
                    let raw = self.token_to_string(&tok);
                    body.push(Statement::Raw(raw));
                }
            }
        }

        Err(ParseError {
            message: "WHILE без ENDWHILE".into(),
            position: self.position,
        })
    }

    /// Аналог parse_while_block, но для вложенных WHILE —
    /// возвращает Result<Statement, ParseError> вместо Result<Option<Statement>, ParseError>.
    fn parse_while_block_inline(&mut self, word: &str) -> Result<Statement, ParseError> {
        let condition = if word.len() > 6 {
            word[6..].to_string()
        } else {
            String::new()
        };

        let mut body = Vec::new();
        let mut depth = 1u32;

        while self.position + 1 < self.tokens.len() {
            self.advance();
            let tok = self.tokens[self.position].clone();

            match &tok {
                Token::Word(w) => {
                    let up = w.to_uppercase();
                    if up == "ENDWHILE" {
                        depth -= 1;
                        if depth == 0 {
                            return Ok(Statement::WhileBlock(WhileStatement {
                                condition,
                                body,
                            }));
                        }
                        body.push(Statement::Word(w.clone()));
                    } else if up.starts_with("WHILE") {
                        // Вложенный WHILE — парсим рекурсивно
                        // depth не увеличиваем: рекурсивный вызов сам обработает
                        // свой ENDWHILE, а внешний ENDWHILE закроет внешний цикл.
                        let inner = self.parse_while_block_inline(w)?;
                        body.push(inner);
                    } else if up.starts_with("IF") {
                        let if_stmt = self.parse_if_block_inline(w)?;
                        body.push(if_stmt);
                    } else {
                        body.push(Statement::Word(w.clone()));
                    }
                }
                Token::NewLine => body.push(Statement::NewLine),
                Token::NCode(code) => body.push(Statement::NCode(*code)),
                Token::GCode(code) => body.push(Statement::Motion(MotionStatement {
                    code: *code,
                    rapid: *code == 0,
                })),
                Token::MCode(code) => {
                    body.push(Statement::Misc(MiscStatement { code: *code }))
                }
                Token::Axis(axis, value) => body.push(Statement::Axis(AxisStatement {
                    axis: axis.clone(),
                    value: *value,
                })),
                Token::AxisExpr(axis, expr) => body.push(Statement::Word(format!(
                    "{}={}",
                    axis, expr
                ))),
                Token::Comment(text) => body.push(Statement::Comment(CommentStatement {
                    text: text.clone(),
                })),
                Token::Eof => {
                    return Err(ParseError {
                        message: "WHILE без ENDWHILE".into(),
                        position: self.position,
                    });
                }
                _ => {
                    let raw = self.token_to_string(&tok);
                    body.push(Statement::Raw(raw));
                }
            }
        }

        Err(ParseError {
            message: "WHILE без ENDWHILE".into(),
            position: self.position,
        })
    }

    fn parse_if_block(&mut self, word: &str) -> Result<Option<Statement>, ParseError> {
        let condition = if word.len() > 3 {
            word[3..].to_string()
        } else {
            String::new()
        };

        let mut then_body = Vec::new();
        let mut else_body: Option<Vec<Statement>> = None;
        let mut in_else = false;
        let mut depth = 1u32;

        while self.position + 1 < self.tokens.len() {
            self.advance();
            let tok = self.tokens[self.position].clone();

            match &tok {
                Token::Word(w) => {
                    let up = w.to_uppercase();
                    if up == "ENDIF" {
                        depth -= 1;
                        if depth == 0 {
                            return Ok(Some(Statement::IfBlock(IfStatement {
                                condition,
                                then_body,
                                else_body,
                            })));
                        }
                        push_to(&mut then_body, &mut else_body, in_else, Statement::Word(w.clone()));
                    } else if up == "ELSE" {
                        if depth == 1 {
                            in_else = true;
                        } else {
                            push_to(&mut then_body, &mut else_body, in_else, Statement::Word(w.clone()));
                        }
                    } else if up.starts_with("IF") {
                        depth += 1;
                        push_to(&mut then_body, &mut else_body, in_else, Statement::Word(w.clone()));
                    } else {
                        push_to(&mut then_body, &mut else_body, in_else, Statement::Word(w.clone()));
                    }
                }
                Token::NewLine => push_to(&mut then_body, &mut else_body, in_else, Statement::NewLine),
                Token::NCode(code) => push_to(&mut then_body, &mut else_body, in_else, Statement::NCode(*code)),
                Token::GCode(code) => push_to(
                    &mut then_body,
                    &mut else_body,
                    in_else,
                    Statement::Motion(MotionStatement {
                        code: *code,
                        rapid: *code == 0,
                    }),
                ),
                Token::MCode(code) => push_to(
                    &mut then_body,
                    &mut else_body,
                    in_else,
                    Statement::Misc(MiscStatement { code: *code }),
                ),
                Token::Axis(axis, value) => push_to(
                    &mut then_body,
                    &mut else_body,
                    in_else,
                    Statement::Axis(AxisStatement {
                        axis: axis.clone(),
                        value: *value,
                    }),
                ),
                Token::AxisExpr(axis, expr) => push_to(
                    &mut then_body,
                    &mut else_body,
                    in_else,
                    Statement::Word(format!("{}={}", axis, expr)),
                ),
                Token::Comment(text) => push_to(
                    &mut then_body,
                    &mut else_body,
                    in_else,
                    Statement::Comment(CommentStatement {
                        text: text.clone(),
                    }),
                ),
                Token::Eof => {
                    return Err(ParseError {
                        message: "IF без ENDIF".into(),
                        position: self.position,
                    });
                }
                _ => {
                    let raw = self.token_to_string(&tok);
                    push_to(&mut then_body, &mut else_body, in_else, Statement::Raw(raw));
                }
            }
        }

        Err(ParseError {
            message: "IF без ENDIF".into(),
            position: self.position,
        })
    }

    fn parse_if_block_inline(&mut self, word: &str) -> Result<Statement, ParseError> {
        let condition = if word.len() > 3 {
            word[3..].to_string()
        } else {
            String::new()
        };

        let mut then_body = Vec::new();
        let mut else_body: Option<Vec<Statement>> = None;
        let mut in_else = false;
        let mut depth = 1u32;

        while self.position + 1 < self.tokens.len() {
            self.advance();
            let tok = self.tokens[self.position].clone();

            match &tok {
                Token::Word(w) => {
                    let up = w.to_uppercase();
                    if up == "ENDIF" {
                        depth -= 1;
                        if depth == 0 {
                            return Ok(Statement::IfBlock(IfStatement {
                                condition,
                                then_body,
                                else_body,
                            }));
                        }
                        push_to(&mut then_body, &mut else_body, in_else, Statement::Word(w.clone()));
                    } else if up == "ELSE" {
                        if depth == 1 {
                            in_else = true;
                        } else {
                            push_to(&mut then_body, &mut else_body, in_else, Statement::Word(w.clone()));
                        }
                    } else if up.starts_with("IF") {
                        depth += 1;
                        push_to(&mut then_body, &mut else_body, in_else, Statement::Word(w.clone()));
                    } else {
                        push_to(&mut then_body, &mut else_body, in_else, Statement::Word(w.clone()));
                    }
                }
                Token::NewLine => push_to(&mut then_body, &mut else_body, in_else, Statement::NewLine),
                Token::NCode(code) => push_to(&mut then_body, &mut else_body, in_else, Statement::NCode(*code)),
                Token::GCode(code) => push_to(
                    &mut then_body,
                    &mut else_body,
                    in_else,
                    Statement::Motion(MotionStatement {
                        code: *code,
                        rapid: *code == 0,
                    }),
                ),
                Token::MCode(code) => push_to(
                    &mut then_body,
                    &mut else_body,
                    in_else,
                    Statement::Misc(MiscStatement { code: *code }),
                ),
                Token::Axis(axis, value) => push_to(
                    &mut then_body,
                    &mut else_body,
                    in_else,
                    Statement::Axis(AxisStatement {
                        axis: axis.clone(),
                        value: *value,
                    }),
                ),
                Token::AxisExpr(axis, expr) => push_to(
                    &mut then_body,
                    &mut else_body,
                    in_else,
                    Statement::Word(format!("{}={}", axis, expr)),
                ),
                Token::Comment(text) => push_to(
                    &mut then_body,
                    &mut else_body,
                    in_else,
                    Statement::Comment(CommentStatement {
                        text: text.clone(),
                    }),
                ),
                Token::Eof => {
                    return Err(ParseError {
                        message: "IF без ENDIF".into(),
                        position: self.position,
                    });
                }
                _ => {
                    let raw = self.token_to_string(&tok);
                    push_to(&mut then_body, &mut else_body, in_else, Statement::Raw(raw));
                }
            }
        }

        Err(ParseError {
            message: "IF без ENDIF".into(),
            position: self.position,
        })
    }

    fn current_token(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn token_to_string(&self, token: &Token) -> String {
        match token {
            Token::GCode(code) => format!("G{}", code),
            Token::MCode(code) => format!("M{}", code),
            Token::Axis(axis, value) => {
                if let Some(v) = value {
                    format!("{}{}", axis, v)
                } else {
                    axis.clone()
                }
            }
            Token::AxisExpr(axis, expr) => format!("{}={}", axis, expr),
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

/// Вспомогательная функция: добавляет Statement в then_body или else_body в зависимости от in_else
fn push_to(
    then_body: &mut Vec<Statement>,
    else_body: &mut Option<Vec<Statement>>,
    in_else: bool,
    stmt: Statement,
) {
    if in_else {
        else_body.get_or_insert_with(Vec::new).push(stmt);
    } else {
        then_body.push(stmt);
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
    }

    #[test]
    fn test_parse_multichar_words() {
        let tokens = tokenize("G64 CFTCP\nMODECHECK(2)");
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.len(), 4);
    }

    #[test]
    fn test_parse_n_codes() {
        let tokens = tokenize("N0100 G0");
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.len(), 2);
        assert_eq!(program[0], Statement::NCode(100));
    }

    #[test]
    fn test_parse_full_input_snapshot() {
        let input = std::fs::read_to_string("input_code.txt")
            .expect("input_code.txt должен существовать в корне проекта");
        let tokens = tokenize(&input);
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        assert!(!program.is_empty());
    }

    #[test]
    fn test_parse_while_block() {
        let tokens = tokenize("WHILE R101<R103\nG1 X10\nENDWHILE");
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::WhileBlock(w) => {
                assert_eq!(w.condition, "R101<R103");
                assert!(!w.body.is_empty());
            }
            _ => panic!("Ожидался WhileBlock"),
        }
    }

    #[test]
    fn test_parse_nested_while() {
        let tokens =
            tokenize("WHILE R101<R103\nWHILE R102<R103\nG1 X10\nENDWHILE\nENDWHILE");
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::WhileBlock(outer) => {
                assert_eq!(outer.condition, "R101<R103");
                assert!(outer
                    .body
                    .iter()
                    .any(|s| matches!(s, Statement::WhileBlock(_))));
            }
            _ => panic!("Ожидался WhileBlock"),
        }
    }

    #[test]
    fn test_parse_if_block() {
        let tokens = tokenize("IF R101==0\nG1 X10\nENDIF");
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::IfBlock(i) => {
                assert_eq!(i.condition, "R101==0");
                assert!(!i.then_body.is_empty());
                assert!(i.else_body.is_none());
            }
            _ => panic!("Ожидался IfBlock"),
        }
    }

    #[test]
    fn test_parse_if_else_block() {
        let tokens = tokenize("IF R101==0\nG1 X10\nELSE\nG1 Y20\nENDIF");
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::IfBlock(i) => {
                assert_eq!(i.condition, "R101==0");
                assert!(i.else_body.is_some());
            }
            _ => panic!("Ожидался IfBlock"),
        }
    }
}
