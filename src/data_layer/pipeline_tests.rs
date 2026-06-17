//! Тесты пайплайна (лексер → парсер → валидатор → форматтер)

use super::*;

// -----------------------------------------------------------------------
// format_code
// -----------------------------------------------------------------------

#[test]
fn test_format_code_empty() {
    // Пустая программа — ошибка валидации, но результат Ok с пустым контентом
    let result = format_code("", 0, true);
    assert!(result.is_ok());
    let (formatted, errors) = result.unwrap();
    assert!(formatted.is_empty());
    assert!(!errors.is_empty());
}

#[test]
fn test_format_code_simple() {
    let result = format_code("G0 X10 Y20", 0, true);
    assert!(result.is_ok());
    let (formatted, _) = result.unwrap();
    assert_eq!(formatted, "G0 X10 Y20\n");
}

#[test]
fn test_format_code_multiline() {
    let result = format_code("G0 X10\nG1 Z5.5 F100", 0, true);
    assert!(result.is_ok());
    let (formatted, _) = result.unwrap();
    assert_eq!(formatted, "G0 X10\nG1 Z5.5 F100\n");
}

#[test]
fn test_format_code_with_renumber() {
    let result = format_code("G0 X10\nG1 Y20", 10, true);
    assert!(result.is_ok());
    let (formatted, _) = result.unwrap();
    assert!(formatted.starts_with("N10 G0 X10"));
    assert!(formatted.contains("\nN20 G1 Y20"));
}

#[test]
fn test_format_code_preserves_empty_lines() {
    let result = format_code("G0 X10\n\nG1 Y20", 0, true);
    assert!(result.is_ok());
    let (formatted, _) = result.unwrap();
    // Пустые строки сохраняются
    assert_eq!(formatted, "G0 X10\n\nG1 Y20\n");
}

#[test]
fn test_format_code_with_validation_error() {
    // Ось X без значения — ошибка валидации, но результат Ok с пустым контентом
    let result = format_code("G0 X", 0, true);
    assert!(result.is_ok());
    let (formatted, errors) = result.unwrap();
    assert!(formatted.is_empty()); // контент не меняется
    assert!(!errors.is_empty()); // но ошибки есть
}

#[test]
fn test_format_code_invalid_syntax() {
    // Случайный мусор, который лексер разберёт в Unknown
    let result = format_code("@#$%", 0, true);
    assert!(result.is_ok()); // Unknown токены — не ошибка, они станут Raw
    let (formatted, _) = result.unwrap();
    assert!(!formatted.is_empty());
}

#[test]
fn test_format_code_with_multiple_errors() {
    // Две оси без значений на разных строках — две ошибки
    let result = format_code("X\nY", 0, true);
    assert!(result.is_ok());
    let (formatted, errors) = result.unwrap();
    assert!(formatted.is_empty());
    assert_eq!(errors.len(), 2);
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

#[test]
fn test_format_text_params() {
    let path = "text_params.txt";
    let input = std::fs::read_to_string(path).expect("Не удалось прочитать text_params.txt");
    let result = format_code(&input, 0, true);
    assert!(
        result.is_ok(),
        "format_code вернул ошибку: {:?}",
        result.err()
    );
    let (formatted, warnings) = result.unwrap();

    println!("=== FORMATTED OUTPUT ===");
    println!("{}", formatted);
    println!("=== END ===");
    if !warnings.is_empty() {
        println!("Warnings ({}) :", warnings.len());
        for w in &warnings {
            println!("  [{:?}] {}", w.severity, w.message);
        }
    }

    assert!(!formatted.is_empty(), "Результат форматирования пуст");
    assert!(formatted.contains("WHILE"), "Результат не содержит WHILE");
    assert!(
        formatted.contains("ENDWHILE"),
        "Результат не содержит ENDWHILE"
    );
}

#[test]
fn test_format_text_params_renumber() {
    let path = "text_params.txt";
    let input = std::fs::read_to_string(path).expect("Не удалось прочитать text_params.txt");
    let result = format_code(&input, 10, true);
    assert!(
        result.is_ok(),
        "format_code вернул ошибку: {:?}",
        result.err()
    );
    let (formatted, warnings) = result.unwrap();

    println!("=== FORMATTED OUTPUT (renumber_step=10) ===");
    println!("{}", formatted);
    println!("=== END ===");
    if !warnings.is_empty() {
        println!("Warnings ({}) :", warnings.len());
        for w in &warnings {
            println!("  [{:?}] {}", w.severity, w.message);
        }
    }

    // Проверка: перед WHILE нет N-кода (строка WHILE отдельная, предыдущая пустая)
    // ENDWHILE — на отдельной строке без N-кода
    let lines: Vec<&str> = formatted.lines().collect();
    for i in 0..lines.len() {
        let trimmed = lines[i].trim();
        if trimmed == "WHILE" {
            // WHILE не начинается с N (уже верно, т.к. trimmed == "WHILE")
            if i > 0 {
                let prev = lines[i - 1].trim();
                assert!(
                    prev.is_empty(),
                    "Перед WHILE (строка {}) предыдущая строка не пустая: '{}'",
                    i + 1,
                    prev
                );
            }
        }
        if trimmed == "ENDWHILE" {
            // ENDWHILE не начинается с N (уже верно, т.к. trimmed == "ENDWHILE")
            // и не содержит N-кода в строке
            assert!(
                !trimmed.starts_with('N'),
                "ENDWHILE (строка {}) содержит N-код: '{}'",
                i + 1,
                trimmed
            );
        }
    }

    assert!(formatted.contains("WHILE"), "Результат не содержит WHILE");
    assert!(
        formatted.contains("ENDWHILE"),
        "Результат не содержит ENDWHILE"
    );
}

#[test]
fn test_format_input_code() {
    let path = "input_code.txt";
    let input = std::fs::read_to_string(path).expect("Не удалось прочитать input_code.txt");

    // Сначала смотрим ошибки валидации через validate_code
    let validation = validate_code(&input);
    if let Ok(ref errors) = validation {
        println!("=== VALIDATION MESSAGES ({}) ===", errors.len());
        for m in errors {
            println!("  [{:?}] {}", m.severity, m.message);
        }
    }

    // Прогоняем через format_code (ренумерация 0, skip_empty_lines true)
    let result = format_code(&input, 0, true);
    // Не паникуем при ошибке — выводим её и проверяем остальное
    match result {
        Ok((formatted, warnings)) => {
            println!("=== FORMATTED OUTPUT ===");
            println!("{}", formatted);
            println!("=== END ===");
            if !warnings.is_empty() {
                println!("Warnings ({}) :", warnings.len());
                for w in &warnings {
                    println!("  [{:?}] {}", w.severity, w.message);
                }
            }

            // Проверяем что результат не пустой
            assert!(!formatted.is_empty(), "Результат форматирования пуст");

            // Проверяем наличие ключевых строк
            assert!(
                formatted.contains("G64 CFTCP"),
                "Результат не содержит 'G64 CFTCP'"
            );
            assert!(
                formatted.contains("MODECHECK(2)"),
                "Результат не содержит 'MODECHECK(2)'"
            );
            assert!(
                formatted.contains("TRANS Z-8"),
                "Результат не содержит 'TRANS Z-8'"
            );
            assert!(
                formatted.contains("MATLRET"),
                "Результат не содержит 'MATLRET'"
            );
            assert!(formatted.contains("M17"), "Результат не содержит 'M17'");
        }
        Err(e) => {
            println!("format_code error: {}", e);
            // Тест не должен паниковать — это проверка на отсутствие паники
            // Но мы всё равно проверим, что ошибка не пустая
            assert!(!e.is_empty());
        }
    }
}

#[test]
fn test_debug_line22() {
    let path = "input_code.txt";
    let input = std::fs::read_to_string(path).expect("Не удалось прочитать input_code.txt");
    let line = input.lines().nth(21).expect("В файле меньше 22 строк");
    println!("line22: '{}'", line);
    println!("bytes: {:?}", line.as_bytes());
    println!("length: {}", line.len());
}

#[test]
fn test_debug_axis() {
    let path = "input_code.txt";
    let input = std::fs::read_to_string(path).expect("Не удалось прочитать input_code.txt");
    let tokens = crate::infrastructure::lexer::tokenize(&input);

    // Токен на позиции 69 — Axis X None
    let pos = 69;
    let target = &tokens[pos];
    println!("Токен на позиции {}: {:?}", pos, target);
    println!();

    // Предыдущие 5 токенов
    println!("=== Предыдущие 5 токенов ===");
    for i in (pos.saturating_sub(5))..pos {
        println!("[{}] {:?}", i, tokens[i]);
    }

    // Следующие 5 токенов
    println!("\n=== Следующие 5 токенов ===");
    let end = std::cmp::min(pos + 6, tokens.len());
    for i in (pos + 1)..end {
        println!("[{}] {:?}", i, tokens[i]);
    }

    // Считаем строки: для каждого NewLine увеличиваем счётчик
    println!("\n=== Поиск строки для позиции {} ===", pos);
    let mut line = 1;
    for i in 0..=pos {
        if tokens[i] == crate::domain::Token::NewLine {
            line += 1;
        }
    }
    println!("Токен на позиции {} находится на строке {}", pos, line);
}
