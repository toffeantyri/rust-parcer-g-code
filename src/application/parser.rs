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
            Token::Axis(axis, value, decimals) => Ok(Some(Statement::Axis(AxisStatement {
                axis: axis.clone(),
                value: *value,
                decimal_places: *decimals,
            }))),
            Token::AxisExpr(axis, expr) => Ok(Some(Statement::Word(format!("{}={}", axis, expr)))),
            Token::Comment(text) => Ok(Some(Statement::Comment(CommentStatement {
                text: text.clone(),
            }))),
            Token::NewLine => Ok(Some(Statement::NewLine)),
            Token::NCode(code) => Ok(Some(Statement::NCode(*code))),
            Token::Speed(val) => Ok(Some(Statement::Speed(val.clone()))),
            Token::RParameter(val) => Ok(Some(Statement::RParameter(val.clone()))),
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
                Token::MCode(code) => body.push(Statement::Misc(MiscStatement { code: *code })),
                Token::Axis(axis, value, decimals) => body.push(Statement::Axis(AxisStatement {
                    axis: axis.clone(),
                    value: *value,
                    decimal_places: *decimals,
                })),
                Token::AxisExpr(axis, expr) => {
                    body.push(Statement::Word(format!("{}={}", axis, expr)))
                }
                Token::Comment(text) => {
                    body.push(Statement::Comment(CommentStatement { text: text.clone() }))
                }
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
                            return Ok(Statement::WhileBlock(WhileStatement { condition, body }));
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
                Token::MCode(code) => body.push(Statement::Misc(MiscStatement { code: *code })),
                Token::Axis(axis, value, decimals) => body.push(Statement::Axis(AxisStatement {
                    axis: axis.clone(),
                    value: *value,
                    decimal_places: *decimals,
                })),
                Token::AxisExpr(axis, expr) => {
                    body.push(Statement::Word(format!("{}={}", axis, expr)))
                }
                Token::Comment(text) => {
                    body.push(Statement::Comment(CommentStatement { text: text.clone() }))
                }
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
                        push_to(
                            &mut then_body,
                            &mut else_body,
                            in_else,
                            Statement::Word(w.clone()),
                        );
                    } else if up == "ELSE" {
                        if depth == 1 {
                            in_else = true;
                        } else {
                            push_to(
                                &mut then_body,
                                &mut else_body,
                                in_else,
                                Statement::Word(w.clone()),
                            );
                        }
                    } else if up.starts_with("IF") {
                        let inner = self.parse_if_block_inline(w)?;
                        push_to(&mut then_body, &mut else_body, in_else, inner);
                    } else if up.starts_with("WHILE") {
                        let inner = self.parse_while_block_inline(w)?;
                        push_to(&mut then_body, &mut else_body, in_else, inner);
                    } else {
                        push_to(
                            &mut then_body,
                            &mut else_body,
                            in_else,
                            Statement::Word(w.clone()),
                        );
                    }
                }
                Token::NewLine => {
                    push_to(&mut then_body, &mut else_body, in_else, Statement::NewLine)
                }
                Token::NCode(code) => push_to(
                    &mut then_body,
                    &mut else_body,
                    in_else,
                    Statement::NCode(*code),
                ),
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
                Token::Axis(axis, value, decimals) => push_to(
                    &mut then_body,
                    &mut else_body,
                    in_else,
                    Statement::Axis(AxisStatement {
                        axis: axis.clone(),
                        value: *value,
                        decimal_places: *decimals,
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
                    Statement::Comment(CommentStatement { text: text.clone() }),
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
                        push_to(
                            &mut then_body,
                            &mut else_body,
                            in_else,
                            Statement::Word(w.clone()),
                        );
                    } else if up == "ELSE" {
                        if depth == 1 {
                            in_else = true;
                        } else {
                            push_to(
                                &mut then_body,
                                &mut else_body,
                                in_else,
                                Statement::Word(w.clone()),
                            );
                        }
                    } else if up.starts_with("IF") {
                        let inner = self.parse_if_block_inline(w)?;
                        push_to(&mut then_body, &mut else_body, in_else, inner);
                    } else if up.starts_with("WHILE") {
                        let inner = self.parse_while_block_inline(w)?;
                        push_to(&mut then_body, &mut else_body, in_else, inner);
                    } else {
                        push_to(
                            &mut then_body,
                            &mut else_body,
                            in_else,
                            Statement::Word(w.clone()),
                        );
                    }
                }
                Token::NewLine => {
                    push_to(&mut then_body, &mut else_body, in_else, Statement::NewLine)
                }
                Token::NCode(code) => push_to(
                    &mut then_body,
                    &mut else_body,
                    in_else,
                    Statement::NCode(*code),
                ),
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
                Token::Axis(axis, value, decimals) => push_to(
                    &mut then_body,
                    &mut else_body,
                    in_else,
                    Statement::Axis(AxisStatement {
                        axis: axis.clone(),
                        value: *value,
                        decimal_places: *decimals,
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
                    Statement::Comment(CommentStatement { text: text.clone() }),
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

    #[allow(dead_code)]
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
            Token::Axis(axis, value, _) => {
                if let Some(v) = value {
                    format!("{}{}", axis, v)
                } else {
                    axis.clone()
                }
            }
            Token::AxisExpr(axis, expr) => format!("{}={}", axis, expr),
            Token::Speed(val) => val.clone(),
            Token::RParameter(val) => val.clone(),
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
#[cfg(test)]
#[path = "parser_tests.rs"]
mod tests;
