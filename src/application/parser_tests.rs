//! Тесты парсера G-кода

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
    let input = std::fs::read_to_string("test_content/input_code.txt")
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
    let tokens = tokenize("WHILE R101<R103\nWHILE R102<R103\nG1 X10\nENDWHILE\nENDWHILE");
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

#[test]
fn test_parse_while_with_inner_if() {
    let tokens = tokenize("WHILE R101<R103\nIF R102==0\nG1 X10\nENDIF\nENDWHILE");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    assert_eq!(program.len(), 1);
    match &program[0] {
        Statement::WhileBlock(w) => {
            assert_eq!(w.condition, "R101<R103");
            assert!(w.body.iter().any(|s| matches!(s, Statement::IfBlock(_))));
        }
        _ => panic!("Ожидался WhileBlock"),
    }
}

#[test]
fn test_parse_if_with_empty_else() {
    let tokens = tokenize("IF R101==0\nG1 X10\nELSE\nENDIF");
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

#[test]
fn test_parse_if_with_inner_while() {
    let tokens = tokenize("IF R101==0\nWHILE R102<R103\nG1 X10\nENDWHILE\nENDIF");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    assert_eq!(program.len(), 1);
    match &program[0] {
        Statement::IfBlock(i) => {
            assert_eq!(i.condition, "R101==0");
            assert!(i
                .then_body
                .iter()
                .any(|s| matches!(s, Statement::WhileBlock(_))));
        }
        _ => panic!("Ожидался IfBlock"),
    }
}

#[test]
fn test_parse_nested_if() {
    let tokens = tokenize("IF R101>0\nIF R102>0\nG1 X10\nENDIF\nENDIF");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    assert_eq!(program.len(), 1);
    match &program[0] {
        Statement::IfBlock(outer) => {
            assert_eq!(outer.condition, "R101>0");
            assert!(outer
                .then_body
                .iter()
                .any(|s| matches!(s, Statement::IfBlock(_))));
        }
        _ => panic!("Ожидался IfBlock"),
    }
}

#[test]
fn test_parse_empty_program() {
    let tokens = vec![];
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    assert!(program.is_empty());
}

#[test]
fn test_parse_while_with_flow_tokens() {
    let tokens = vec![
        Token::WhileBlock("R101<R103".to_string()),
        Token::GCode(1),
        Token::NewLine,
        Token::EndWhile,
    ];
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    assert_eq!(program.len(), 1);
    match &program[0] {
        Statement::WhileBlock(w) => {
            assert_eq!(w.condition, "R101<R103");
            assert_eq!(w.body.len(), 2);
        }
        _ => panic!("Expected WhileBlock"),
    }
}

#[test]
fn test_parse_if_with_flow_tokens() {
    let tokens = vec![
        Token::IfBlock("R101==0".to_string()),
        Token::GCode(0),
        Token::NewLine,
        Token::EndIf,
    ];
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    assert_eq!(program.len(), 1);
    match &program[0] {
        Statement::IfBlock(i) => {
            assert_eq!(i.condition, "R101==0");
            assert_eq!(i.then_body.len(), 2);
            assert!(i.else_body.is_none());
        }
        _ => panic!("Expected IfBlock"),
    }
}

#[test]
fn test_parse_speed_tokens() {
    let tokens = tokenize("S1000 F200");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    assert_eq!(program.len(), 2);
    assert_eq!(program[0], Statement::Speed("S1000".to_string()));
    assert_eq!(program[1], Statement::Speed("F200".to_string()));
}

#[test]
fn test_parse_while_nested_flow_tokens() {
    let tokens = vec![
        Token::WhileBlock("R1<R2".to_string()),
        Token::WhileBlock("R3<R4".to_string()),
        Token::GCode(1),
        Token::NewLine,
        Token::EndWhile,
        Token::NewLine,
        Token::EndWhile,
    ];
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    assert_eq!(program.len(), 1);
    match &program[0] {
        Statement::WhileBlock(outer) => {
            assert_eq!(outer.body.len(), 2);
            match &outer.body[0] {
                Statement::WhileBlock(inner) => {
                    assert_eq!(inner.body.len(), 2);
                }
                _ => panic!("Expected inner WhileBlock"),
            }
        }
        _ => panic!("Expected outer WhileBlock"),
    }
}

#[test]
fn test_parse_comment_token() {
    let tokens = tokenize("; just a comment");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    assert_eq!(program.len(), 1);
    assert_eq!(
        program[0],
        Statement::Comment(CommentStatement {
            text: " just a comment".to_string()
        })
    );
}
