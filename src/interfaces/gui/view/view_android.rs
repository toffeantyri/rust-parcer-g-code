//! View для Android — минимальная версия.
//! Будет расширена при сборке под Android.

use std::sync::mpsc;
use std::time::Instant;

use egui::TextStyle;

use crate::data_layer::EditorCommand;
use crate::infrastructure::highlight::build_highlighted_job;
use crate::interfaces::gui::intent::Intent;
use crate::interfaces::gui::model::Model;

/// Заглушка сборки интентов для Android.
pub fn collect_intents(_ctx: &egui::Context, _is_busy: bool, _model: &Model) -> Vec<Intent> {
    Vec::new()
}

/// Отрисовывает редактор кода.
pub fn view_editor(
    model: &mut Model,
    ctx: &egui::Context,
    _cmd_tx: &mpsc::Sender<EditorCommand>,
    last_text_change: &mut Instant,
    pending_text: &mut Option<String>,
) {
    let content_before = model.content().to_string();
    let mut edit_content = model.content().to_string();

    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical()
            .id_salt("editor_scroll")
            .show(ui, |ui| {
                ui.add_sized(
                    ui.available_size(),
                    egui::TextEdit::multiline(&mut edit_content)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .desired_rows(50)
                        .font(TextStyle::Monospace)
                        .layouter(&mut |_ui: &egui::Ui, text: &str, _wrap_width: f32| {
                            let job = build_highlighted_job(text, model.error_lines(), None);
                            _ui.fonts(|f| f.layout_job(job))
                        }),
                );
            });
    });

    if edit_content != content_before {
        model.set_content(edit_content);
        model.set_modified(true);
        *pending_text = Some(model.content().to_string());
        *last_text_change = Instant::now();
    }
}

pub fn view_statusbar(model: &Model, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.label(model.status());
    });
}

pub fn view_settings(_model: &Model, _ctx: &egui::Context) -> Vec<Intent> {
    Vec::new()
}

pub fn view_exit_dialog(_model: &Model, _ctx: &egui::Context) -> Vec<Intent> {
    Vec::new()
}

pub fn view_search_dialog(_model: &mut Model, _ctx: &egui::Context) -> Vec<Intent> {
    Vec::new()
}

pub fn view_replace_dialog(_model: &mut Model, _ctx: &egui::Context) -> Vec<Intent> {
    Vec::new()
}

#[allow(dead_code)]
pub fn view_axis_swap_dialog(_model: &mut Model, _ctx: &egui::Context) -> Vec<Intent> {
    Vec::new()
}

pub fn view_shortcuts(_model: &Model, _ctx: &egui::Context) -> Vec<Intent> {
    Vec::new()
}
