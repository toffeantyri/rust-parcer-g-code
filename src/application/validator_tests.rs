use super::*;
use crate::domain::*;
use crate::shared::Severity;

#[test]
fn test_validate_empty_program() {
    let msgs = validate(&[]);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].severity, Severity::Error);
    assert_eq!(msgs[0].line, 0);
}

#[test]
fn test_validate_axis_without_value() {
    let program = vec![Statement::Axis(AxisStatement {
        axis: "X".to_string(),
        value: None,
        decimal_places: None,
    })];
    let msgs = validate(&program);
    assert_eq!(msgs.len(), 1);
    assert!(msgs[0].message.contains("X"));
    // Первая строка — line=1
    assert_eq!(msgs[0].line, 1);
}

#[test]
fn test_validate_axis_without_value_on_line_3() {
    let program = vec![
        Statement::NewLine,
        Statement::NewLine,
        Statement::Axis(AxisStatement {
            axis: "Y".to_string(),
            value: None,
            decimal_places: None,
        }),
    ];
    let msgs = validate(&program);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].line, 3);
}

#[test]
fn test_validate_ok_program() {
    let program = vec![
        Statement::Motion(MotionStatement {
            code: 0,
            rapid: true,
        }),
        Statement::Axis(AxisStatement {
            axis: "X".to_string(),
            value: Some(10.0),
            decimal_places: None,
        }),
        Statement::NewLine,
    ];
    let msgs = validate(&program);
    assert!(msgs.is_empty());
}

#[test]
fn test_validate_axis_expr_without_value() {
    // X= без значения — должно быть ошибкой
    let program = vec![Statement::Word("X=".to_string())];
    let msgs = validate(&program);
    assert_eq!(msgs.len(), 1);
    assert!(msgs[0].message.contains("X"));
}

#[test]
fn test_validate_axis_expr_with_value_is_ok() {
    // X=10 — это нормальное выражение, не ошибка
    let program = vec![Statement::Word("X=10".to_string())];
    let msgs = validate(&program);
    assert!(msgs.is_empty());
}

#[test]
fn test_validate_axis_expr_other_letter_is_not_error() {
    // D= — D не ось в G-коде, не должно быть ошибкой
    let program = vec![Statement::Word("D=".to_string())];
    let msgs = validate(&program);
    assert!(msgs.is_empty());
}

#[test]
fn test_validate_axis_expr_without_value_via_pipeline() {
    // Проверяем через валидацию полного пайплайна
    let program = vec![Statement::Word("X=".to_string()), Statement::NewLine];
    let msgs = validate(&program);
    assert_eq!(msgs.len(), 1);
}

#[test]
fn test_validate_axis_expr_empty_via_tokenize() {
    // X= из лексера — должен давать ошибку валидации
    let tokens = crate::infrastructure::lexer::tokenize("X=");
    let mut parser = crate::application::Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    let msgs = validate(&program);
    assert_eq!(msgs.len(), 1);
    assert!(msgs[0].message.contains("X"));
}

// -----------------------------------------------------------------------
// Подозрительные двухбуквенные слова
// -----------------------------------------------------------------------

#[test]
fn test_validate_suspicious_xx_is_warning() {
    let program = vec![Statement::Word("XX".to_string())];
    let msgs = validate(&program);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].severity, Severity::Warning);
    assert!(msgs[0].message.contains("XX"));
}

#[test]
fn test_validate_suspicious_xx20_is_warning() {
    let tokens = crate::infrastructure::lexer::tokenize("XX20");
    let mut parser = crate::application::Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    let msgs = validate(&program);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].severity, Severity::Warning);
}

#[test]
fn test_validate_suspicious_xz_is_warning() {
    let program = vec![Statement::Word("XZ".to_string())];
    let msgs = validate(&program);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].severity, Severity::Warning);
}

#[test]
fn test_validate_normal_word_is_not_warning() {
    let program = vec![Statement::Word("MODECHECK".to_string())];
    let msgs = validate(&program);
    assert!(msgs.is_empty());
}

#[test]
fn test_validate_axis_with_value_is_not_warning() {
    let program = vec![Statement::Axis(AxisStatement {
        axis: "X".to_string(),
        value: Some(10.0),
        decimal_places: None,
    })];
    let msgs = validate(&program);
    assert!(msgs.is_empty());
}
