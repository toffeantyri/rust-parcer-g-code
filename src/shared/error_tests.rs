//! Тесты типов ошибок (ValidationMessage, ParseError)

use super::*;

#[test]
fn test_error_with_line() {
    let msg = ValidationMessage::error(5, "ось X без значения");
    let s = msg.to_string();
    assert!(s.contains("ОШИБКА"));
    assert!(s.contains("[строка 5]"));
    assert!(s.contains("ось X без значения"));
}

#[test]
fn test_warning_without_line() {
    let msg = ValidationMessage::warning(0, "проверьте оси");
    let s = msg.to_string();
    assert!(s.contains("ПРЕДУПРЕЖДЕНИЕ"));
    assert!(!s.contains("[строка"));
    assert!(s.contains("проверьте оси"));
}

#[test]
fn test_error_with_line_zero() {
    let msg = ValidationMessage::error(0, "программа пуста");
    let s = msg.to_string();
    assert!(s.contains("ОШИБКА"));
    assert!(!s.contains("[строка"));
    assert_eq!(msg.severity, Severity::Error);
    assert_eq!(msg.line, 0);
}

#[test]
fn test_parse_error_display() {
    let err = ParseError {
        message: "неизвестный токен".to_string(),
        position: 42,
    };
    let s = err.to_string();
    assert!(s.contains("ParseError"));
    assert!(s.contains("42"));
    assert!(s.contains("неизвестный токен"));
}
