//! View — отрисовка UI, возвращает намерения

use eframe::egui;

use super::intent::Intent;
use super::model::Model;

/// Собирает намерения от UI: меню + панель инструментов.
pub fn collect_intents(ctx: &egui::Context, is_busy: bool) -> Vec<Intent> {
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
                // Анимированный индикатор: добавляем точки в зависимости от времени
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

/// Отрисовывает редактор кода. Требует `&mut model.content`.
pub fn view_editor(model: &mut Model, ctx: &egui::Context) {
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
}
