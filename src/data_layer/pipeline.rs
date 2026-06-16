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
#[path = "pipeline_tests.rs"]
mod tests;
