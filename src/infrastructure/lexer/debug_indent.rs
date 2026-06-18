//! Debug: проверка отступов при ПЕРЕНУМЕРАЦИИ
#[test]
fn test_debug_indent() {
    let input = "G0 X10\nWHILE R1<R2\n  Z=R3\n  ENDWHILE\nG1 Y20";

    let tokens = crate::infrastructure::lexer::tokenize(input);
    let mut parser = crate::application::Parser::new(tokens);
    let program = parser.parse_program().unwrap();

    // Без перенумерации
    let config1 = crate::application::FormatConfig {
        uppercase_codes: true,
        decimal_places: 5,
        renumber_step: 0,
        skip_empty_lines: true,
        ..Default::default()
    };
    let fmt1 = crate::application::Formatter::new(config1);
    let formatted1 = fmt1.format_program(&program);
    eprintln!("WITHOUT renumbering:\n{}", formatted1);

    // С перенумерацией
    let config2 = crate::application::FormatConfig {
        uppercase_codes: true,
        decimal_places: 5,
        renumber_step: 10,
        skip_empty_lines: true,
        ..Default::default()
    };
    let fmt2 = crate::application::Formatter::new(config2);
    let formatted2 = fmt2.format_program(&program);
    eprintln!("WITH renumbering:\n{}", formatted2);
    panic!("done");
}
