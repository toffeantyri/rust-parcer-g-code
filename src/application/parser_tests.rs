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
