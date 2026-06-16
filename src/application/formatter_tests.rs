//! Тесты форматтера G-кода

use super::*;
use crate::domain::*;

#[test]
fn test_format_simple_program() {
    let program = vec![
        Statement::Motion(MotionStatement { code: 0, rapid: true }),
        Statement::Axis(AxisStatement { axis: "X".to_string(), value: Some(10.0) }),
        Statement::Axis(AxisStatement { axis: "Y".to_string(), value: Some(20.0) }),
        Statement::NewLine,
        Statement::Motion(MotionStatement { code: 1, rapid: false }),
        Statement::Axis(AxisStatement { axis: "Z".to_string(), value: Some(5.5) }),
        Statement::Axis(AxisStatement { axis: "F".to_string(), value: Some(100.0) }),
    ];
    let formatter = Formatter::new(FormatConfig::default());
    let result = formatter.format_program(&program);
    assert_eq!(result, "G0 X10.00000 Y20.00000\nG1 Z5.50000 F100.00000\n");
}

#[test]
fn test_format_ncode() {
    let program = vec![
        Statement::NCode(100),
        Statement::Motion(MotionStatement { code: 0, rapid: true }),
        Statement::NewLine,
        Statement::NCode(105),
        Statement::Motion(MotionStatement { code: 1, rapid: false }),
    ];
    let formatter = Formatter::new(FormatConfig::default());
    let result = formatter.format_program(&program);
    assert_eq!(result, "N0100 G0\nN0105 G1\n");
}

#[test]
fn test_format_with_raw() {
    let program = vec![
        Statement::Motion(MotionStatement { code: 64, rapid: false }),
        Statement::Raw("CFTCP".to_string()),
        Statement::NewLine,
        Statement::Raw("MODECHECK".to_string()),
        Statement::Comment(CommentStatement { text: "2".to_string() }),
        Statement::NewLine,
        Statement::Raw("MAMILL".to_string()),
        Statement::NewLine,
        Statement::Motion(MotionStatement { code: 0, rapid: true }),
    ];
    let formatter = Formatter::new(FormatConfig::default());
    let result = formatter.format_program(&program);
    assert!(result.contains("G64"));
    assert!(result.contains("CFTCP"));
}

#[test]
fn test_format_empty_lines() {
    let program = vec![
        Statement::NewLine,
        Statement::NewLine,
        Statement::Motion(MotionStatement { code: 0, rapid: true }),
    ];
    let formatter = Formatter::new(FormatConfig::default());
    let result = formatter.format_program(&program);
    assert_eq!(result, "\n\nG0\n");
}

#[test]
fn test_renumber_step() {
    let program = vec![
        Statement::NCode(100),
        Statement::Motion(MotionStatement { code: 0, rapid: true }),
        Statement::NewLine,
        Statement::NewLine,
        Statement::NCode(999),
        Statement::Motion(MotionStatement { code: 1, rapid: false }),
    ];
    let config = FormatConfig { renumber_step: 10, skip_empty_lines: true, ..Default::default() };
    let formatter = Formatter::new(config);
    let result = formatter.format_program(&program);
    assert_eq!(result, "N10 G0\n\nN20 G1\n");
}

#[test]
fn test_renumber_skip_empty_false() {
    let program = vec![
        Statement::NCode(1),
        Statement::Motion(MotionStatement { code: 0, rapid: true }),
        Statement::NewLine,
        Statement::NewLine,
        Statement::NCode(2),
        Statement::Motion(MotionStatement { code: 1, rapid: false }),
    ];
    let config = FormatConfig { renumber_step: 1, skip_empty_lines: false, ..Default::default() };
    let formatter = Formatter::new(config);
    let result = formatter.format_program(&program);
    assert_eq!(result, "N1 G0\nN2\nN3 G1\n");
}

#[test]
fn test_renumber_adds_ncode_to_lines_without() {
    let program = vec![
        Statement::Motion(MotionStatement { code: 0, rapid: true }),
        Statement::Axis(AxisStatement { axis: "X".to_string(), value: Some(10.0) }),
        Statement::NewLine,
        Statement::Motion(MotionStatement { code: 1, rapid: false }),
    ];
    let config = FormatConfig { renumber_step: 1, skip_empty_lines: true, ..Default::default() };
    let formatter = Formatter::new(config);
    let result = formatter.format_program(&program);
    assert!(result.starts_with("N1 G0"));
    assert!(result.contains("\nN2 G1"));
}

#[test]
fn test_renumber_removes_empty_ncode_lines() {
    let program = vec![
        Statement::NCode(100),
        Statement::NewLine,
        Statement::NCode(200),
        Statement::Motion(MotionStatement { code: 0, rapid: true }),
    ];
    let config = FormatConfig { renumber_step: 10, skip_empty_lines: true, ..Default::default() };
    let formatter = Formatter::new(config);
    let result = formatter.format_program(&program);
    assert_eq!(result, "\nN20 G0\n");
}

#[test]
fn test_format_while_block() {
    let program = vec![Statement::WhileBlock(WhileStatement {
        condition: "R101<R103".into(),
        body: vec![
            Statement::Motion(MotionStatement { code: 1, rapid: false }),
            Statement::Axis(AxisStatement { axis: "X".into(), value: Some(10.0) }),
            Statement::NewLine,
        ],
    })];
    let formatter = Formatter::new(FormatConfig::default());
    let result = formatter.format_program(&program);
    assert!(result.contains("WHILE R101<R103\n"));
    assert!(result.contains("  G1 X10.00000\n"));
    assert!(result.contains("ENDWHILE\n"));
}

#[test]
fn test_format_while_with_ncode() {
    // N-код на отдельной строке, WHILE без номера
    let program = vec![
        Statement::NCode(230),
        Statement::WhileBlock(WhileStatement {
            condition: "R101<R103".into(),
            body: vec![
                Statement::Motion(MotionStatement { code: 1, rapid: false }),
                Statement::NewLine,
            ],
        }),
    ];
    let formatter = Formatter::new(FormatConfig::default());
    let result = formatter.format_program(&program);
    assert!(result.contains("N0230\nWHILE R101<R103\n"), "N-код должен быть на отдельной строке перед WHILE:\n{}", result);
    assert!(result.contains("  G1\n"));
    assert!(result.contains("ENDWHILE\n"));
}

#[test]
fn test_format_if_else_block() {
    let program = vec![Statement::IfBlock(IfStatement {
        condition: "R101==0".into(),
        then_body: vec![
            Statement::Motion(MotionStatement { code: 0, rapid: true }),
            Statement::Axis(AxisStatement { axis: "X".into(), value: Some(10.0) }),
            Statement::NewLine,
        ],
        else_body: Some(vec![
            Statement::Motion(MotionStatement { code: 1, rapid: false }),
            Statement::Axis(AxisStatement { axis: "Y".into(), value: Some(20.0) }),
            Statement::NewLine,
        ]),
    })];
    let formatter = Formatter::new(FormatConfig::default());
    let result = formatter.format_program(&program);
    assert!(result.contains("IF R101==0\n"));
    assert!(result.contains("  G0 X10.00000\n"));
    assert!(result.contains("ELSE\n"));
    assert!(result.contains("  G1 Y20.00000\n"));
    assert!(result.contains("ENDIF\n"));
}

#[test]
fn test_format_nested_while() {
    let program = vec![Statement::WhileBlock(WhileStatement {
        condition: "R101<R103".into(),
        body: vec![
            Statement::WhileBlock(WhileStatement {
                condition: "R102<R103".into(),
                body: vec![
                    Statement::Motion(MotionStatement { code: 1, rapid: false }),
                    Statement::Axis(AxisStatement { axis: "X".into(), value: Some(10.0) }),
                    Statement::NewLine,
                ],
            }),
            Statement::NewLine,
        ],
    })];
    let formatter = Formatter::new(FormatConfig::default());
    let result = formatter.format_program(&program);
    assert!(result.contains("WHILE R101<R103\n"));
    assert!(result.contains("  WHILE R102<R103\n"));
    assert!(result.contains("    G1 X10.00000\n"));
    assert!(result.contains("  ENDWHILE\n"));
    assert!(result.contains("ENDWHILE\n"));
}

#[test]
fn test_format_if_without_else() {
    let program = vec![Statement::IfBlock(IfStatement {
        condition: "R101==0".into(),
        then_body: vec![
            Statement::Motion(MotionStatement { code: 0, rapid: true }),
            Statement::NewLine,
        ],
        else_body: None,
    })];
    let formatter = Formatter::new(FormatConfig::default());
    let result = formatter.format_program(&program);
    assert!(result.contains("IF R101==0\n"));
    assert!(result.contains("  G0\n"));
    assert!(result.contains("ENDIF\n"));
    assert!(!result.contains("ELSE"));
}

#[test]
fn test_format_nested_if() {
    let program = vec![Statement::IfBlock(IfStatement {
        condition: "R101>0".into(),
        then_body: vec![
            Statement::IfBlock(IfStatement {
                condition: "R102>0".into(),
                then_body: vec![
                    Statement::Motion(MotionStatement { code: 1, rapid: false }),
                    Statement::NewLine,
                ],
                else_body: None,
            }),
            Statement::NewLine,
        ],
        else_body: None,
    })];
    let formatter = Formatter::new(FormatConfig::default());
    let result = formatter.format_program(&program);
    assert!(result.contains("IF R101>0\n"));
    assert!(result.contains("  IF R102>0\n"));
    assert!(result.contains("    G1\n"));
    assert!(result.contains("  ENDIF\n"));
    assert!(result.contains("ENDIF\n"));
}

#[test]
fn test_format_with_tabs() {
    let program = vec![Statement::WhileBlock(WhileStatement {
        condition: "R101<R103".into(),
        body: vec![
            Statement::Motion(MotionStatement { code: 1, rapid: false }),
            Statement::NewLine,
        ],
    })];
    let config = FormatConfig { use_spaces: false, indent_size: 2, ..Default::default() };
    let formatter = Formatter::new(config);
    let result = formatter.format_program(&program);
    assert!(result.contains("WHILE R101<R103\n"));
    assert!(result.contains("\tG1\n"));
    assert!(result.contains("ENDWHILE\n"));
}

#[test]
fn test_format_empty_while() {
    let program = vec![Statement::WhileBlock(WhileStatement {
        condition: "TRUE".into(),
        body: vec![],
    })];
    let formatter = Formatter::new(FormatConfig::default());
    let result = formatter.format_program(&program);
    assert_eq!(result, "WHILE TRUE\nENDWHILE\n");
}
