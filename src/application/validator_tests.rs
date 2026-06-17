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
