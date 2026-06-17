//! Прикладной слой: валидатор — проверяет AST на синтаксические ошибки
//!
//! Проверки:
//! - Ось без значения (например одинокий `X` без числа)
//! - Ось с `=` без значения после равно (например `X=`)
//! - Подозрительные двухбуквенные слова (например `XX`, `XZ`, `XX=`)
//! - Пустая программа

use crate::domain::Statement;
use crate::shared::{Severity, ValidationMessage};

/// Буквы осей G-кода (должен совпадать с лексером)
const AXIS_LETTERS: &str = "XYZABCUVWFIJK";

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
            Statement::Speed(s) => {
                if s.trim().is_empty() || s == "S" || s == "S=" {
                    messages.push(ValidationMessage::error(
                        line,
                        "Скорость шпинделя (S) указана без значения".to_string(),
                    ));
                }
            }
            Statement::RParameter(r) => {
                // RParameter должен быть непустым
                if r.len() <= 1 || r == "R" || r == "R=" {
                    messages.push(ValidationMessage::warning(
                        line,
                        "R-параметр без значения".to_string(),
                    ));
                }
            }
            Statement::Word(word) => {
                // Проверяем оси с `=` и пустым значением, например `X=`
                // Word вида "X=" (одна буква + =) — ошибка
                if word.len() == 2 && word.ends_with('=') {
                    let axis = &word[..1];
                    if AXIS_LETTERS.contains(axis) {
                        messages.push(ValidationMessage::error(
                            line,
                            format!("Ось '{}' указана без значения", axis),
                        ));
                    }
                // Подозрительные двухбуквенные слова, где первая буква — ось
                // Например "XX", "XY", "XZ" — скорее всего опечатка
                // Не проверяем если слово уже дало ошибку ("X=" и т.п.)
                } else if word.len() == 2 && AXIS_LETTERS.contains(&word[..1]) {
                    messages.push(ValidationMessage::warning(
                        line,
                        format!("Подозрительная конструкция '{}'. Возможно, опечатка.", word),
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
