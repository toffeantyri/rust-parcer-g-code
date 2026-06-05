//! Интерфейсный слой: CLI-обработчик программы
//!
//! Читает файл, передаёт данные в application через infrastructure.

use std::fs;
use std::process;

use crate::application::{FormatConfig, Formatter, Parser};
use crate::infrastructure::Lexer;

/// Запускает полный цикл: чтение файла -> лексинг -> парсинг -> форматирование -> вывод
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
    let mut lexer = Lexer::new(input_content);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        if token == crate::domain::Token::Eof {
            break;
        }
        tokens.push(token);
    }

    // Парсинг: токены -> AST
    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(prog) => prog,
        Err(e) => {
            eprintln!("Ошибка парсинга: {}", e);
            process::exit(1);
        }
    };

    // Форматирование: AST -> строка
    let formatter = Formatter::new(FormatConfig::default());
    let formatted = formatter.format_program(&program);

    // Вывод результата
    print!("{}", formatted);
}
