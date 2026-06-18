//! View — отрисовка UI, возвращает намерения

use std::sync::mpsc;
use std::time::Instant;

use egui::TextStyle;

use crate::data_layer::EditorCommand;

use crate::shared::i18n;

use crate::domain::Token;
use crate::infrastructure::lexer::tokenize_with_positions;

use crate::interfaces::gui::intent::Intent;
use crate::interfaces::gui::model::Model;

/// Собирает намерения от UI: меню + панель инструментов.
pub fn collect_intents(ctx: &egui::Context, is_busy: bool, model: &Model) -> Vec<Intent> {
    let mut intents = Vec::new();

    egui::TopBottomPanel::top("menu_panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button(&i18n::locale().menu.file, |ui| {
                if ui
                    .add_enabled(!is_busy, egui::Button::new(&i18n::locale().menu.open))
                    .clicked()
                {
                    intents.push(Intent::OpenFile);
                    ui.close_menu();
                }
                if ui
                    .add_enabled(!is_busy, egui::Button::new(&i18n::locale().menu.save))
                    .clicked()
                {
                    intents.push(Intent::SaveFile);
                    ui.close_menu();
                }
                if ui
                    .add_enabled(!is_busy, egui::Button::new(&i18n::locale().menu.save_as))
                    .clicked()
                {
                    intents.push(Intent::SaveAs);
                    ui.close_menu();
                }
                if ui.button(&i18n::locale().menu.close).clicked() {
                    intents.push(Intent::CloseFile);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(&i18n::locale().menu.exit).clicked() {
                    intents.push(Intent::Exit);
                    ui.close_menu();
                }
            });
            ui.menu_button(&i18n::locale().menu.edit, |ui| {
                if ui
                    .add_enabled(!is_busy, egui::Button::new(&i18n::locale().menu.format))
                    .clicked()
                {
                    intents.push(Intent::Format);
                    ui.close_menu();
                }
                if ui
                    .add_enabled(!is_busy, egui::Button::new(&i18n::locale().menu.validate))
                    .clicked()
                {
                    intents.push(Intent::Validate);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(&i18n::locale().menu.format_settings).clicked() {
                    intents.push(Intent::ToggleSettings);
                    ui.close_menu();
                }
            });
            ui.menu_button(&i18n::locale().menu.settings, |ui| {
                let is_ru = model.format_settings().language == "ru";
                if ui
                    .selectable_label(is_ru, &i18n::locale().menu.lang_ru)
                    .clicked()
                    && !is_ru
                {
                    intents.push(Intent::SetLanguage("ru".to_string()));
                }
                let is_en = model.format_settings().language == "en";
                if ui
                    .selectable_label(is_en, &i18n::locale().menu.lang_en)
                    .clicked()
                    && !is_en
                {
                    intents.push(Intent::SetLanguage("en".to_string()));
                }
            });
            ui.menu_button(&i18n::locale().menu.help, |ui| {
                if ui.button(&i18n::locale().menu.shortcuts).clicked() {
                    intents.push(Intent::ToggleShortcuts);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(&i18n::locale().menu.about).clicked() {
                    ui.close_menu();
                }
            });
        });
    });

    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!is_busy, egui::Button::new(&i18n::locale().toolbar.open))
                .clicked()
            {
                intents.push(Intent::OpenFile);
            }
            if ui
                .add_enabled(!is_busy, egui::Button::new(&i18n::locale().toolbar.save))
                .clicked()
            {
                intents.push(Intent::SaveFile);
            }
            if ui
                .add_enabled(!is_busy, egui::Button::new(&i18n::locale().toolbar.format))
                .clicked()
            {
                intents.push(Intent::Format);
            }
            if ui
                .add_enabled(!is_busy, egui::Button::new(&i18n::locale().toolbar.check))
                .clicked()
            {
                intents.push(Intent::Validate);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(model.file_path())
                        .size(12.0)
                        .color(egui::Color32::GRAY),
                );
            });
        });
    });

    intents
}

/// Отрисовывает строку состояния.
pub fn view_statusbar(model: &Model, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let mut status = model.status().to_string();
            if model.is_busy() {
                let dots = ((ctx.input(|i| i.time) * 4.0) as usize) % 4;
                status.push(' ');
                for i in 0..3 {
                    if i < dots {
                        status.push('.');
                    } else {
                        status.push(' ');
                    }
                }
            }
            ui.label(egui::RichText::new(&status).size(12.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new("v0.1")
                        .size(11.0)
                        .color(egui::Color32::GRAY),
                );
            });
        });
    });
}

/// Отрисовывает окно настроек форматирования.
pub fn view_settings(model: &Model, ctx: &egui::Context) -> Vec<Intent> {
    let mut intents = Vec::new();
    let mut open_copy = model.settings_open();

    if !open_copy {
        return intents;
    }

    egui::Window::new(&i18n::locale().settings.title)
        .open(&mut open_copy)
        .resizable(false)
        .default_size([320.0, 200.0])
        .show(ctx, |ui| {
            ui.label(&i18n::locale().settings.renumber_step);
            ui.horizontal(|ui| {
                for &step in &[1u32, 10, 100] {
                    let selected = model.format_settings().renumber_step == step;
                    if ui.selectable_label(selected, format!("{}", step)).clicked() {
                        intents.push(Intent::SetRenumberStep(step));
                    }
                }
            });
            ui.add_space(8.0);
            ui.label(&i18n::locale().settings.examples);
            let step = model.format_settings().renumber_step;
            ui.label(format!("N{} N{} N{} ...", step, step * 2, step * 3));
            ui.add_space(12.0);
            let mut skip = model.format_settings().skip_empty_lines;
            if ui
                .checkbox(&mut skip, &i18n::locale().settings.skip_empty)
                .changed()
            {
                intents.push(Intent::SetSkipEmptyLines(skip));
            }
        });

    // Если окно было закрыто через крестик — синхронизируем
    if !open_copy {
        intents.push(Intent::ToggleSettings);
    }

    intents
}

/// Отрисовывает редактор кода и отслеживает изменения.
pub fn view_editor(
    model: &mut Model,
    ctx: &egui::Context,
    _cmd_tx: &mpsc::Sender<EditorCommand>,
    last_text_change: &mut Instant,
    pending_text: &mut Option<String>,
) {
    let content_before = model.content().to_string();
    // Извлекаем String из модели для TextEdit (который требует &mut String)
    let mut edit_content = model.content().to_string();

    egui::CentralPanel::default().show(ctx, |ui| {
        let editor_id = ui.next_auto_id();

        // Если файл только что загружен — фокусируем редактор
        if model.editor_needs_focus() {
            ctx.memory_mut(|mem| mem.request_focus(editor_id));
            model.set_editor_needs_focus(false);
        }

        egui::ScrollArea::vertical()
            .id_salt("editor_scroll")
            .show(ui, |ui| {
                ui.add_sized(
                    ui.available_size(),
                    egui::TextEdit::multiline(&mut edit_content)
                        .id(editor_id)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .desired_rows(50)
                        .font(TextStyle::Monospace)
                        .layouter(&mut |_ui: &egui::Ui, text: &str, _wrap_width: f32| {
                            let job = build_highlighted_job(text, model.error_lines());
                            _ui.fonts(|f| f.layout_job(job))
                        }),
                );
            });
    });

    // Если текст изменился — помечаем как modified и отправляем TextChanged с coalesce
    if edit_content != content_before {
        model.set_content(edit_content);
        model.set_modified(true);
        *pending_text = Some(model.content().to_string());
        *last_text_change = Instant::now();
    }
}

/// Строит LayoutJob с подсветкой синтаксиса для G-кода.
fn build_highlighted_job(text: &str, error_lines: &[usize]) -> egui::text::LayoutJob {
    use egui::Color32;

    let mut job = egui::text::LayoutJob::default();
    let mut tokens = tokenize_with_positions(text);

    // Сортируем по позиции (на случай если лексер вернёт не по порядку)
    tokens.sort_by_key(|t| t.start);

    // Цвета для разных типов токенов
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

    fn line_bg(line_num: usize, error_lines: &[usize]) -> Color32 {
        if error_lines.contains(&line_num) {
            Color32::from_rgba_premultiplied(200, 0, 0, 40)
        } else {
            Color32::TRANSPARENT
        }
    }

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

/// Отрисовывает диалог подтверждения выхода/закрытия.
pub fn view_exit_dialog(model: &Model, ctx: &egui::Context) -> Vec<Intent> {
    let mut intents = Vec::new();
    if !model.show_exit_dialog() {
        return intents;
    }

    // Esc → отмена
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        intents.push(Intent::CancelAction);
        return intents;
    }

    let mut is_open = true;
    egui::Window::new(&i18n::locale().dialog.exit_title)
        .open(&mut is_open)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([350.0, 120.0])
        .show(ctx, |ui| {
            ui.label(&i18n::locale().dialog.confirm_save);
            ui.add_space(12.0);

            // Состояние выбранной кнопки (для навигации стрелками)
            let btn_id = ui.id().with("exit_btn_idx");
            let mut selected: usize = ctx.data_mut(|d| d.get_temp::<usize>(btn_id).unwrap_or(1));

            // Стрелки для переключения кнопок
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight)) {
                selected = (selected + 1).min(2);
                ctx.data_mut(|d| d.insert_temp(btn_id, selected));
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft)) {
                selected = selected.saturating_sub(1);
                ctx.data_mut(|d| d.insert_temp(btn_id, selected));
            }

            // Enter — подтвердить выбранную кнопку
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
                match selected {
                    0 => intents.push(Intent::ConfirmSave),
                    1 => intents.push(Intent::DiscardAndContinue),
                    _ => intents.push(Intent::CancelAction),
                }
            }

            ui.horizontal(|ui| {
                let labels = [
                    &i18n::locale().dialog.btn_save,
                    &i18n::locale().dialog.btn_discard,
                    &i18n::locale().dialog.btn_cancel,
                ];
                let actions = [
                    Intent::ConfirmSave,
                    Intent::DiscardAndContinue,
                    Intent::CancelAction,
                ];

                for (i, label) in labels.iter().enumerate() {
                    let is_selected = i == selected;
                    let btn = egui::Button::new(*label).min_size(egui::vec2(100.0, 30.0));
                    let response = if is_selected {
                        ui.add(btn.stroke(egui::Stroke::new(2.0, egui::Color32::WHITE)))
                    } else {
                        ui.add(btn)
                    };
                    if response.clicked() {
                        intents.push(actions[i].clone());
                    }
                }
            });
        });

    // Если закрыли крестиком — считаем отменой
    if !is_open && model.show_exit_dialog() {
        intents.push(Intent::CancelAction);
    }

    intents
}

/// Отрисовывает окно горячих клавиш.
pub fn view_shortcuts(model: &Model, ctx: &egui::Context) -> Vec<Intent> {
    let mut intents = Vec::new();
    if !model.shortcuts_open() {
        return intents;
    }

    let mut open_copy = true;
    egui::Window::new("⌨ ".to_string() + &i18n::locale().menu.shortcuts)
        .open(&mut open_copy)
        .resizable(false)
        .default_size([420.0, 300.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            use egui::RichText;

            let shortcuts = [
                ("Ctrl+O", &i18n::locale().menu.open),
                ("Ctrl+S", &i18n::locale().menu.save),
                ("Ctrl+Shift+S", &i18n::locale().menu.save_as),
                ("F5", &i18n::locale().menu.format),
                ("F6", &i18n::locale().menu.validate),
            ];

            ui.label(
                RichText::new(&i18n::locale().menu.shortcuts_title)
                    .size(16.0)
                    .strong(),
            );
            ui.separator();
            ui.add_space(8.0);

            for (key, desc) in &shortcuts {
                ui.horizontal(|ui| {
                    ui.monospace(*key);
                    ui.label("  —  ");
                    ui.label(*desc);
                });
            }
        });

    if !open_copy {
        intents.push(Intent::ToggleShortcuts);
    }

    intents
}
