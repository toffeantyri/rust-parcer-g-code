//! Application — точка входа egui-android-framework.
//!
//! Собирает все слои вместе: Component, Data Layer, StateStore.
//! Реализует trait Application из egui-android-framework.

use std::sync::mpsc;

use egui_android_framework::store::StateStore;
use egui_android_framework::{
    AndroidWakeHandle, AppConfig, Application, Component, LifecycleObserver, UiNotifier,
};

use crate::interfaces::gui::integration::component::GCodeComponent;
use crate::interfaces::gui::integration::data_layer;
use crate::interfaces::gui::integration::msg::Msg;
use crate::interfaces::gui::integration::state::AppState;

impl LifecycleObserver for GCodeApp {}

/// Главное приложение G-Code Editor под Android.
pub struct GCodeApp {
    root: GCodeComponent,
    config: AppConfig,
    cmd_tx: mpsc::Sender<Msg>,
    notify_rx: mpsc::Receiver<()>,
}

impl GCodeApp {
    /// Создаёт приложение со всеми зависимостями.
    fn new() -> Self {
        let config = AppConfig::default();
        let store = StateStore::new(AppState::default());

        let (cmd_tx, cmd_rx) = mpsc::channel::<Msg>();
        let (notify_tx, notify_rx) = mpsc::channel::<()>();

        // Data Layer в фоновом потоке
        let store_for_worker = store.clone_state();
        data_layer::spawn_data_layer(cmd_rx, store_for_worker, notify_tx);

        // Component с доступом к StateStore
        let root = GCodeComponent::new(store.clone_state());

        Self {
            root,
            config,
            cmd_tx,
            notify_rx,
        }
    }
}

impl Application for GCodeApp {
    type RootComponent = GCodeComponent;

    fn create() -> Self {
        Self::new()
    }

    fn root(&mut self) -> &mut GCodeComponent {
        &mut self.root
    }

    fn root_ref(&self) -> &GCodeComponent {
        &self.root
    }

    fn config(&self) -> &AppConfig {
        &self.config
    }

    fn config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    fn create_notifier(
        &mut self,
        ctx: &egui::Context,
        wake: AndroidWakeHandle,
    ) -> Option<UiNotifier> {
        let rx = std::mem::replace(&mut self.notify_rx, mpsc::channel().1);
        Some(UiNotifier::new(ctx.clone(), Some(wake), rx))
    }

    fn frame(&mut self, egui_ctx: &egui::Context, raw_input: egui::RawInput) -> egui::FullOutput {
        // 1. Корректируем DPI: на Android плотность ~2.5-3.5x.
        //    Ставим pixels_per_point = 1.5 для комфортного размера.
        egui_ctx.set_pixels_per_point(3.0);

        // 2. Синхронизируем снепшот с последним состоянием из Store
        self.root.sync_from_store();

        // 3. Рендерим UI и собираем сообщения
        let mut messages = Vec::new();
        let output = egui_ctx.run(raw_input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                messages = self.root.render(ui);
            });
        });

        // 4. Обрабатываем сообщения: логируем + отправляем в Data Layer
        for msg in messages {
            self.root.handle(msg.clone());
            let _ = self.cmd_tx.send(msg);
        }

        output
    }
}
