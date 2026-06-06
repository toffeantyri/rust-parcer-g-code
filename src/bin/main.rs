//! Точка входа CLI для парсера и форматтера G-кода
//!
//! Использует interfaces::run для полного цикла обработки.

use std::env;

fn main() {
    // Чтение пути к файлу из аргументов командной строки
    let args: Vec<String> = env::args().collect();
    let input_path = if args.len() > 1 {
        &args[1]
    } else {
        "input_code.txt"
    };

    println!("🚀 Запуск парсера и форматтера G-кода...");
    println!("Файл: {}\n", input_path);

    // Запуск полного цикла через интерфейсный слой
    code_parser::interfaces::run(input_path);
}
