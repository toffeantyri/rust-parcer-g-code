//! Pipeline — ресурсоёмкие операции: лексер → парсер → валидатор → форматтер.

use crate::shared::{Severity, ValidationMessage};

/// Форматирует G-код: лексинг → парсинг → валидация → форматирование.
/// Возвращает (отформатированная строка, ошибки валидации).
pub fn format_code(
    input: &str,
    renumber_step: u32,
    skip_empty_lines: bool,
) -> Result<(String, Vec<ValidationMessage>), String> {
    let tokens = crate::infrastructure::lexer::tokenize(input);
    let mut parser = crate::application::Parser::new(tokens);
    let program = parser
        .parse_program()
        .map_err(|e| format!("Ошибка парсинга: {}", e))?;

    let errors = crate::application::validate(&program);
    let has_errors = errors.iter().any(|e| e.severity == Severity::Error);

    if has_errors {
        return Err(format!(
            "Найдено {} ошибок. Форматирование отменено.",
            errors.len()
        ));
    }

    let config = crate::application::FormatConfig {
        uppercase_codes: true,
        decimal_places: 5,
        renumber_step,
        skip_empty_lines,
        ..Default::default()
    };
    let fmt = crate::application::Formatter::new(config);
    let formatted = fmt.format_program(&program);

    Ok((formatted, errors))
}

/// Проверяет G-код: лексинг → парсинг → валидация.
pub fn validate_code(input: &str) -> Result<Vec<ValidationMessage>, String> {
    let tokens = crate::infrastructure::lexer::tokenize(input);
    let mut parser = crate::application::Parser::new(tokens);
    let program = parser
        .parse_program()
        .map_err(|e| format!("Ошибка парсинга: {}", e))?;

    Ok(crate::application::validate(&program))
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // format_code
    // -----------------------------------------------------------------------

    #[test]
    fn test_format_code_empty() {
        // Пустая программа — ошибка валидации
        let result = format_code("", 0, true);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ошибок"));
    }

    #[test]
    fn test_format_code_simple() {
        let result = format_code("G0 X10 Y20", 0, true);
        assert!(result.is_ok());
        let (formatted, _) = result.unwrap();
        assert_eq!(formatted, "G0 X10.00000 Y20.00000\n");
    }

    #[test]
    fn test_format_code_multiline() {
        let result = format_code("G0 X10\nG1 Z5.5 F100", 0, true);
        assert!(result.is_ok());
        let (formatted, _) = result.unwrap();
        assert_eq!(formatted, "G0 X10.00000\nG1 Z5.50000 F100.00000\n");
    }

    #[test]
    fn test_format_code_with_renumber() {
        let result = format_code("G0 X10\nG1 Y20", 10, true);
        assert!(result.is_ok());
        let (formatted, _) = result.unwrap();
        assert!(formatted.starts_with("N10 G0 X10.00000"));
        assert!(formatted.contains("\nN20 G1 Y20.00000"));
    }

    #[test]
    fn test_format_code_preserves_empty_lines() {
        let result = format_code("G0 X10\n\nG1 Y20", 0, true);
        assert!(result.is_ok());
        let (formatted, _) = result.unwrap();
        // Пустые строки сохраняются
        assert_eq!(formatted, "G0 X10.00000\n\nG1 Y20.00000\n");
    }

    #[test]
    fn test_format_code_with_validation_error() {
        // Ось X без значения — ошибка валидации
        let result = format_code("G0 X", 0, true);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ошибок"));
    }

    #[test]
    fn test_format_code_invalid_syntax() {
        // Случайный мусор, который лексер разберёт в Unknown
        let result = format_code("@#$%", 0, true);
        assert!(result.is_ok()); // Unknown токены — не ошибка, они станут Raw
        let (formatted, _) = result.unwrap();
        assert!(!formatted.is_empty());
    }

    // -----------------------------------------------------------------------
    // validate_code
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_code_ok() {
        let result = validate_code("G0 X10 Y20");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_validate_code_empty() {
        let result = validate_code("");
        assert!(result.is_ok());
        let errors = result.unwrap();
        // Пустая программа — это ошибка валидации
        assert!(!errors.is_empty());
        assert_eq!(errors[0].severity, Severity::Error);
    }

    #[test]
    fn test_validate_code_axis_without_value() {
        let result = validate_code("X");
        assert!(result.is_ok());
        let errors = result.unwrap();
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("X"));
    }

    #[test]
    fn test_validate_code_multichar_words() {
        // Многосимвольные команды не должны вызывать ошибок
        let result = validate_code("MODECHECK(2) TRANS CFTCP");
        assert!(result.is_ok());
        let errors = result.unwrap();
        // Если нет оси без значения — ошибок быть не должно
        let has_axis_errors = errors.iter().any(|e| e.severity == Severity::Error);
        assert!(!has_axis_errors);
    }
}
