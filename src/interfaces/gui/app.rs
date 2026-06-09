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

        // 1. View → Intent: собираем намерения от UI
        let intents = view::collect_intents(ctx, is_busy, &self.model.file_path);

        // 2. Intent → Update: применяем каждое намерение к модели
        for intent in &intents {
            self.model.apply(intent);
        }

        // 2b. Settings view (отдельный проход, т.к. Window может вернуть intents)
        let settings_intents = view::view_settings(&self.model, ctx);
        for intent in &settings_intents {
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
