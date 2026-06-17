//! Прикладной слой: валидатор — проверяет AST на синтаксические ошибки
//!
//! Проверки:
//! - Ось без значения (например одинокий `X` без числа)
//! - Пустая программа

use crate::domain::Statement;
use crate::shared::ValidationMessage;

/// Проверяет программу (AST) на синтаксические ошибки.
/// Возвращает список сообщений валидации с номерами строк.
pub fn validate(program: &[Statement]) -> Vec<ValidationMessage> {
    let mut messages = Vec::new();

    if program.is_empty() {
        messages.push(ValidationMessage::error(0, "Программа пуста"));
        return messages;
    }

    // Номер текущей строки (1-based)
    let mut line: usize = 1;

    for stmt in program {
        match stmt {
            Statement::NewLine => {
                line += 1;
            }
            Statement::Axis(a) => {
                if a.value.is_none() {
                    messages.push(ValidationMessage::error(
                        line,
                        format!("Ось '{}' указана без значения", a.axis),
                    ));
                }
            }
            _ => {}
        }
    }

    messages
}

#[cfg(test)]
#[path = "validator_tests.rs"]
mod tests;
