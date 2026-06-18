//! Подсветка синтаксиса G-кода для GUI.
//!
//! Содержит функцию `build_highlighted_job`, которая строит `egui::text::LayoutJob`
//! с расцветкой токенов. Вынесена из `interfaces` в `infrastructure`,
//! чтобы убрать прямую зависимость `interfaces` от `domain::Token`.

use crate::domain::Token;
use crate::infrastructure::lexer::tokenize_with_positions;
use egui::Color32;

/// Строит LayoutJob с подсветкой синтаксиса для G-кода.
pub fn build_highlighted_job(text: &str, error_lines: &[usize]) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let mut tokens = tokenize_with_positions(text);

    // Сортируем по позиции (на случай если лексер вернёт не по порядку)
    tokens.sort_by_key(|t| t.start);

    let mut current_pos = 0;
    let mut current_line: usize = 1;

    for tp in &tokens {
        // Если есть пропуск между токенами (пробелы) — добавляем их без подсветки
        if tp.start > current_pos {
            let gap = &text[current_pos..tp.start];
            // Считаем строки в пропуске
            for c in gap.chars() {
                if c == '\n' {
                    current_line += 1;
                }
            }
            job.append(
                gap,
                0.0,
                egui::TextFormat {
                    background: line_bg(current_line, error_lines),
                    ..Default::default()
                },
            );
        }

        // Подсветка самого токена
        let token_text = &text[tp.start..tp.end];
        // Считаем переносы строк внутри токена (для NewLine)
        if tp.token == Token::NewLine {
            current_line += 1;
        }
        job.append(
            token_text,
            0.0,
            egui::TextFormat {
                color: token_color(&tp.token),
                background: line_bg(current_line, error_lines),
                ..Default::default()
            },
        );

        current_pos = tp.end;
    }

    // Если после последнего токена есть остаток текста
    if current_pos < text.len() {
        let remaining = &text[current_pos..];
        job.append(
            remaining,
            0.0,
            egui::TextFormat {
                background: line_bg(current_line, error_lines),
                ..Default::default()
            },
        );
    }

    job
}

/// Возвращает цвет для токена на основе его типа.
fn token_color(token: &Token) -> Color32 {
    match token {
        // Зелёный — салатовый
        Token::GCode(_) => Color32::from_rgb(120, 210, 100),
        // Синий — чуть темнее
        Token::MCode(_) => Color32::from_rgb(50, 120, 200),
        Token::Speed(_) => Color32::from_rgb(135, 206, 250),
        Token::RParameter(_) => Color32::from_rgb(30, 80, 160),
        // Жёлтый — светлее
        Token::Axis(_, _, _) => Color32::from_rgb(220, 210, 80),
        Token::AxisExpr(_, _) => Color32::from_rgb(180, 150, 30),
        Token::Comment(_) => Color32::from_rgb(140, 140, 140),
        Token::Word(w) => {
            let upper = w.to_uppercase();
            if upper == "WHILE"
                || upper == "IF"
                || upper == "ELSE"
                || upper == "ENDWHILE"
                || upper == "ENDIF"
                || upper == "REPEAT"
                || upper == "UNTIL"
            {
                Color32::from_rgb(200, 100, 100)
            // R-параметры — тёмно-синий (только R + цифры)
            } else if (w.starts_with('R') || w.starts_with('r'))
                && w.len() > 1
                && w[1..].chars().next().is_some_and(|c| c.is_ascii_digit())
            {
                Color32::from_rgb(30, 80, 160)
            } else {
                Color32::from_rgb(200, 80, 80)
            }
        }
        Token::Unknown(_) => Color32::from_rgb(200, 50, 50),
        _ => Color32::WHITE,
    }
}

/// Возвращает цвет фона для строки (красный полупрозрачный, если есть ошибка).
fn line_bg(line_num: usize, error_lines: &[usize]) -> Color32 {
    if error_lines.contains(&line_num) {
        Color32::from_rgba_premultiplied(200, 0, 0, 40)
    } else {
        Color32::TRANSPARENT
    }
}
