//! View — отрисовка UI, возвращает намерения

use std::sync::mpsc;
use std::time::Instant;

use eframe::egui;

use crate::data_layer::EditorCommand;

use super::intent::Intent;
use super::model::Model;

/// Собирает намерения от UI: меню + панель инструментов.
pub fn collect_intents(ctx: &egui::Context, is_busy: bool, file_path: &str) -> Vec<Intent> {
    let mut intents = Vec::new();

    egui::TopBottomPanel::top("menu_panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui
                    .add_enabled(!is_busy, egui::Button::new("Open..."))
                    .clicked()
                {
                    intents.push(Intent::OpenFile);
                    ui.close_menu();
                }
                if ui
                    .add_enabled(!is_busy, egui::Button::new("Save"))
                    .clicked()
                {
                    intents.push(Intent::SaveFile);
                    ui.close_menu();
                }
                if ui
                    .add_enabled(!is_busy, egui::Button::new("Save as..."))
                    .clicked()
                {
                    intents.push(Intent::SaveAs);
                    ui.close_menu();
                }
                if ui.button("Close").clicked() {
                    intents.push(Intent::CloseFile);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    intents.push(Intent::Exit);
                    ui.close_menu();
                }
            });
            ui.menu_button("Edit", |ui| {
                if ui
                    .add_enabled(!is_busy, egui::Button::new("Format (F5)"))
                    .clicked()
                {
                    intents.push(Intent::Format);
                    ui.close_menu();
                }
                if ui
                    .add_enabled(!is_busy, egui::Button::new("Validate (F6)"))
                    .clicked()
                {
                    intents.push(Intent::Validate);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Format Settings...").clicked() {
                    intents.push(Intent::ToggleSettings);
                    ui.close_menu();
                }
            });
            ui.menu_button("Help", |ui| {
                if ui.button("Shortcuts").clicked() {
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("About").clicked() {
                    ui.close_menu();
                }
            });
        });
    });

    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!is_busy, egui::Button::new("Open"))
                .clicked()
            {
                intents.push(Intent::OpenFile);
            }
            if ui
                .add_enabled(!is_busy, egui::Button::new("Save"))
                .clicked()
            {
                intents.push(Intent::SaveFile);
            }
            if ui
                .add_enabled(!is_busy, egui::Button::new("Format"))
                .clicked()
            {
                intents.push(Intent::Format);
            }
            if ui
                .add_enabled(!is_busy, egui::Button::new("Check"))
                .clicked()
            {
                intents.push(Intent::Validate);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(file_path)
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

    egui::Window::new("Format Settings")
        .open(&mut open_copy)
        .resizable(false)
        .default_size([320.0, 200.0])
        .show(ctx, |ui| {
            ui.label("Renumber step:");
            ui.horizontal(|ui| {
                for &step in &[1u32, 10, 100] {
                    let selected = model.format_settings.renumber_step == step;
                    if ui.selectable_label(selected, format!("{}", step)).clicked() {
                        intents.push(Intent::SetRenumberStep(step));
                    }
                }
            });
            ui.add_space(8.0);
            ui.label("Examples:");
            let step = model.format_settings.renumber_step;
            ui.label(format!("N{} N{} N{} ...", step, step * 2, step * 3));
            ui.add_space(12.0);
            let mut skip = model.format_settings.skip_empty_lines;
            if ui
                .checkbox(&mut skip, "Skip empty lines when renumbering")
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
                        .font(egui::TextStyle::Monospace),
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
    egui::Window::new("Save changes?")
        .open(&mut is_open)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([350.0, 120.0])
        .show(ctx, |ui| {
            ui.label("Do you want to save changes?");
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    intents.push(Intent::ConfirmSave);
                }
                if ui.button("Discard").clicked() {
                    intents.push(Intent::DiscardAndContinue);
                }
                if ui.button("Cancel").clicked() {
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
