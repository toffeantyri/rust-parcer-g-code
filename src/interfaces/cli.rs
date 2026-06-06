//! Интерфейсный слой: CLI-обработчик программы
//!
//! Читает файл, передаёт данные в application через infrastructure.

use std::fs;
use std::process;

use crate::application::{validate, FormatConfig, Formatter, Parser};
use crate::infrastructure::lexer::tokenize;
use crate::shared::Severity;

/// Запускает полный цикл: чтение файла -> лексинг -> парсинг -> валидация -> форматирование -> вывод
pub fn run(input_path: &str) {
    // Чтение входного файла (infrastructure — работа с файловой системой)
    let input_content = match fs::read_to_string(input_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Ошибка чтения файла '{}': {}", input_path, e);
            process::exit(1);
        }
    };

    if cfg!(debug_assertions) {
        println!("Контент файла: \n{}", input_content)
    }

    // Лексинг: текст -> токены
    let tokens = tokenize(&input_content);

    // Парсинг: токены -> AST
    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(prog) => prog,
        Err(e) => {
            eprintln!("Ошибка парсинга: {}", e);
            process::exit(1);
        }
    };

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
        eprintln!();
        eprintln!("Найдены критические ошибки. Форматирование отменено.");
        process::exit(1);
    }

    // Форматирование: AST -> строка
    let formatter = Formatter::new(FormatConfig::default());
    let formatted = formatter.format_program(&program);

    // Вывод результата
    print!("{}", formatted);
}
