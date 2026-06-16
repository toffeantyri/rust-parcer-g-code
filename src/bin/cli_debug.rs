//! Отладочный CLI: читает файл G-кода и выводит результат лексера.
//! Использование: cargo run --bin debug-tokens -- <путь_к_файлу>
//! Если путь не указан, читает text_params.txt из корня проекта.

use std::env;
use std::fs;
use std::io::{self, Write};

use code_parser::infrastructure::lexer::tokenize;
use code_parser::domain::Token;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 {
        args[1].clone()
    } else {
        "text_params.txt".to_string()
    };

    let input = fs::read_to_string(&path)
        .unwrap_or_else(|e| {
            eprintln!("Ошибка чтения файла '{}': {}", path, e);
            std::process::exit(1);
        });

    let lines: Vec<&str> = input.lines().collect();

    // ── ИСХОДНЫЙ ТЕКСТ ──────────────────────────────────────────────
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  Файл: {:<48} ║", truncate(&path, 48));
    println!("║  Строк: {:<46} ║", lines.len());
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║              ИСХОДНЫЙ ТЕКСТ                             ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    for (i, line) in lines.iter().enumerate() {
        let num = i + 1;
        if line.trim().is_empty() {
            println!("  {num:>4} | (пустая строка)");
        } else {
            println!("  {num:>4} | {line}");
        }
    }

    let tokens = tokenize(&input);

    // ── ТОКЕНЫ ЛЕКСЕРА ─────────────────────────────────────────────
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║              ТОКЕНЫ ЛЕКСЕРА                             ║");
    println!("╠══════╤══════════════════════╤════════════════════════════╣");
    println!("║  №   │ Тип                 │ Значение                   ║");
    println!("╠══════╪══════════════════════╪════════════════════════════╣");

    for (i, tok) in tokens.iter().enumerate() {
        let (type_name, value) = format_token(tok);
        println!("║ {:<4} │ {:<20} │ {:<26} ║", i, truncate(&type_name, 20), truncate(&value, 26));
    }

    println!("╚══════╧══════════════════════╧════════════════════════════╝");
    println!("  Всего токенов: {}", tokens.len());

    // ── ТЕКСТ, ВОССТАНОВЛЕННЫЙ ИЗ ТОКЕНОВ ──────────────────────────
    println!();
    println!("══════════════════════════════════════════════════════════════");
    println!("  ТЕКСТ ПОСЛЕ ЛЕКСЕРА (восстановлен из токенов)");
    println!("──────────────────────────────────────────────────────────────");
    println!("  Легенда:");
    println!("    <NL>      перевод строки");
    println!("    [Word]    многосимвольное слово / параметр");
    println!("     Axis     ось со значением");
    println!("     Axis=    ось с выражением");
    println!("     (G/M/N)  коды");
    println!("     ?ch?     неизвестный символ");
    println!("──────────────────────────────────────────────────────────────");
    println!();

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    for tok in &tokens {
        write_token(&mut handle, tok);
    }
    let _ = handle.write_all(b"\n");
    let _ = handle.flush();
}

/// Форматирует токен в пару (тип, значение) для табличного вывода
fn format_token(tok: &Token) -> (String, String) {
    match tok {
        Token::GCode(n) => ("GCode".to_string(), format!("G{n:02}")),
        Token::MCode(n) => ("MCode".to_string(), format!("M{n:02}")),
        Token::NCode(n) => ("NCode".to_string(), format!("N{n:04}")),
        Token::Word(s) => ("Word".to_string(), s.clone()),
        Token::Axis(letter, val) => {
            let v = val.map(|v| format!("{v}")).unwrap_or_else(|| "None".to_string());
            ("Axis".to_string(), format!("{letter} = {v}"))
        }
        Token::AxisExpr(letter, expr) => {
            ("AxisExpr".to_string(), format!("{letter}={expr}"))
        }
        Token::Number(n) => ("Number".to_string(), format!("{n}")),
        Token::Comment(s) => ("Comment".to_string(), s.clone()),
        Token::NewLine => ("NewLine".to_string(), "\\n".to_string()),
        Token::Eof => ("Eof".to_string(), "".to_string()),
        Token::Unknown(ch) => ("Unknown".to_string(), format!("'{ch}'")),
    }
}

/// Выводит токен в человекопонятном виде — как он будет выглядеть в тексте
fn write_token(handle: &mut dyn Write, tok: &Token) {
    let _ = match tok {
        Token::GCode(n) => write!(handle, "G{n:02} "),
        Token::MCode(n) => write!(handle, "M{n:02} "),
        Token::NCode(n) => write!(handle, "N{n:04} "),
        Token::Word(s) => {
            if s.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) {
                write!(handle, "{s} ")
            } else {
                write!(handle, "[{s}] ")
            }
        }
        Token::Axis(letter, val) => {
            match val {
                Some(v) => write!(handle, "{letter}{v} "),
                None => write!(handle, "{letter}? "),
            }
        }
        Token::AxisExpr(letter, expr) => {
            write!(handle, "{letter}={expr} ")
        }
        Token::Number(n) => write!(handle, "{n} "),
        Token::Comment(s) => write!(handle, ";{s}"),
        Token::NewLine => writeln!(handle, "<NL>"),
        Token::Eof => Ok(()),
        Token::Unknown(ch) => write!(handle, "?{ch}? "),
    };
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        format!("{}...", &s[..max - 3])
    } else {
        s.to_string()
    }
}
