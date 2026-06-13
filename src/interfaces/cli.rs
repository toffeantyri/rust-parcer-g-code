//! Интерфейсный слой: CLI-обработчик программы
//!
//! Читает файл, передаёт данные в application через infrastructure.

use std::process;

use anyhow::Context;

use crate::application::{validate, FormatConfig, Formatter, Parser};
use crate::infrastructure::lexer::tokenize;
use crate::shared::Severity;

/// Запускает полный цикл: чтение файла -> лексинг -> парсинг -> валидация -> форматирование -> вывод
pub fn run(input_path: &str) {
    if let Err(e) = run_inner(input_path) {
        eprintln!("{:#}", e);
        process::exit(1);
    }
}

/// Внутренняя функция с anyhow-обработкой ошибок
fn run_inner(input_path: &str) -> anyhow::Result<()> {
    // Чтение входного файла с контекстом
    let input_content =
        std::fs::read_to_string(input_path).context("Не удалось прочитать входной файл")?;

    if cfg!(debug_assertions) {
        println!("Контент файла: \n{}", input_content)
    }

    // Лексинг: текст -> токены
    let tokens = tokenize(&input_content);

    // Парсинг: токены -> AST
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().context("Ошибка парсинга G-кода")?;

    // Валидация: проверка AST на ошибки
    let validation_messages = validate(&program);
    let has_errors = validation_messages
        .iter()
        .any(|m| m.severity == Severity::Error);

    for msg in &validation_messages {
        eprintln!("{}", msg);
    }

    // Если есть критические ошибки — не выводим результат
    if has_errors {
        anyhow::bail!("Найдены критические ошибки. Форматирование отменено.");
    }

    // Форматирование: AST -> строка
    let formatter = Formatter::new(FormatConfig::default());
    let formatted = formatter.format_program(&program);

    // Вывод результата
    print!("{}", formatted);

    Ok(())
}
