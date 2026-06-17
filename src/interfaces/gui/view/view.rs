//! View — отрисовка UI, возвращает намерения

use std::sync::mpsc;
use std::time::Instant;

use egui::{text::LayoutJob, Color32, TextStyle};

use crate::data_layer::EditorCommand;

use crate::shared::i18n;

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
                let is_ru = model.format_settings.language == "ru";
                if ui
                    .selectable_label(is_ru, &i18n::locale().menu.lang_ru)
                    .clicked()
                    && !is_ru
                {
                    intents.push(Intent::SetLanguage("ru".to_string()));
                }
                let is_en = model.format_settings.language == "en";
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
                    egui::RichText::new(&model.file_path)
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
            let mut status = model.status.clone();
            if model.is_busy {
                let dots = ((ctx.input(|i| i.time) * 4.0) as usize) % 4;
                status.push_str(" ");
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
    let mut open_copy = model.settings_open;

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
                    let selected = model.format_settings.renumber_step == step;
                    if ui.selectable_label(selected, format!("{}", step)).clicked() {
                        intents.push(Intent::SetRenumberStep(step));
                    }
                }
            });
            ui.add_space(8.0);
            ui.label(&i18n::locale().settings.examples);
            let step = model.format_settings.renumber_step;
            ui.label(format!("N{} N{} N{} ...", step, step * 2, step * 3));
            ui.add_space(12.0);
            let mut skip = model.format_settings.skip_empty_lines;
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
    let content_before = model.content.clone();

    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical()
            .id_salt("editor_scroll")
            .show(ui, |ui| {
                ui.add_sized(
                    ui.available_size(),
                    egui::TextEdit::multiline(&mut model.content)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .desired_rows(50)
                        .font(TextStyle::Monospace)
                        .layouter(&mut |ui: &egui::Ui, text: &str, _wrap_width: f32| {
                            let mut job = LayoutJob::default();
                            let error_lines = &model.error_lines;
                            let mut char_offset = 0;

                            for (i, _line) in text.split('\n').enumerate() {
                                let line_num = i + 1;
                                let bg = if error_lines.contains(&line_num) {
                                    Color32::from_rgba_premultiplied(200, 0, 0, 40)
                                } else {
                                    Color32::TRANSPARENT
                                };

                                let line_end = text[char_offset..]
                                    .find('\n')
                                    .map(|pos| char_offset + pos)
                                    .unwrap_or(text.len());
                                let line_str = &text[char_offset..line_end];

                                job.append(
                                    line_str,
                                    0.0,
                                    egui::TextFormat {
                                        background: bg,
                                        ..Default::default()
                                    },
                                );

                                // Добавляем перенос строки, если это не последняя строка
                                // или если текст заканчивается на \n
                                if line_end < text.len() {
                                    job.append(
                                        "\n",
                                        0.0,
                                        egui::TextFormat {
                                            background: Color32::TRANSPARENT,
                                            ..Default::default()
                                        },
                                    );
                                    char_offset = line_end + 1;
                                } else {
                                    // Достигли конца текста — следующей итерации не будет
                                    break;
                                }
                            }

                            ui.fonts(|f| f.layout_job(job))
                        }),
                );
            });
    });

    // Если текст изменился — помечаем как modified и отправляем TextChanged с coalesce
    if model.content != content_before {
        model.modified = true;
        *pending_text = Some(model.content.clone());
        *last_text_change = Instant::now();
    }
}

/// Отрисовывает диалог подтверждения выхода/закрытия.
pub fn view_exit_dialog(model: &Model, ctx: &egui::Context) -> Vec<Intent> {
    let mut intents = Vec::new();
    if !model.show_exit_dialog {
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
            ui.horizontal(|ui| {
                if ui.button(&i18n::locale().dialog.btn_save).clicked() {
                    intents.push(Intent::ConfirmSave);
                }
                if ui.button(&i18n::locale().dialog.btn_discard).clicked() {
                    intents.push(Intent::DiscardAndContinue);
                }
                if ui.button(&i18n::locale().dialog.btn_cancel).clicked() {
                    intents.push(Intent::CancelAction);
                }
            });
        });

    // Если закрыли крестиком — считаем отменой
    if !is_open && model.show_exit_dialog {
        intents.push(Intent::CancelAction);
    }

    intents
}

/// Отрисовывает окно горячих клавиш.
pub fn view_shortcuts(model: &Model, ctx: &egui::Context) -> Vec<Intent> {
    let mut intents = Vec::new();
    if !model.shortcuts_open {
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
