//! App — точка входа eframe, соединяет Model, View и Update

use eframe::egui;

use super::model::Model;
use super::view;

/// Главное приложение G-Code Editor.
pub struct GCodeApp {
    model: Model,
}

impl Default for GCodeApp {
    fn default() -> Self {
        let mut model = Model {
            status: "Готов к работе. Откройте файл G-кода.".to_string(),
            ..Default::default()
        };
        model.load_settings();
        Self { model }
    }
}

impl eframe::App for GCodeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Если приложение занято (диалог открыт) — блокируем кнопки
        let is_busy = self.model.is_busy;

        // Проверяем, не был ли запрошен выход через крестик окна
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.model.modified && !self.model.file_path.is_empty() {
                // Отменяем закрытие и показываем диалог
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.model.show_exit_dialog = true;
                self.model.pending_action = Some(super::model::PendingAction::Exit);
            }
            // Если modified = false — окно закроется автоматически
        }

        // 1. View → Intent: собираем намерения от UI
        let intents = view::collect_intents(ctx, is_busy, &self.model.file_path);

        // 2. Intent → Update: применяем каждое намерение к модели
        for intent in &intents {
            self.model.apply(intent);
        }

        // 2b. Settings view
        let settings_intents = view::view_settings(&self.model, ctx);
        for intent in &settings_intents {
            self.model.apply(intent);
        }

        // 2c. Exit dialog
        let exit_intents = view::view_exit_dialog(&self.model, ctx);
        for intent in &exit_intents {
            self.model.apply(intent);
        }

        // 3. View: отображаем статусбар и редактор
        view::view_statusbar(&self.model, ctx);
        view::view_editor(&mut self.model, ctx);

        // Запрашиваем перерисовку только когда нужна анимация спиннера
        if self.model.is_busy {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}
