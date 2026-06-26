//! Component — обёртка над View для egui-android-framework.
//!
//! Хранит снепшот состояния, каждым кадром синхронизируется с StateStore.
//! Не мутирует state, не знает о data layer напрямую.

use egui_android_framework::{store::StateStore, Component};

use crate::interfaces::gui::integration::msg::Msg;
use crate::interfaces::gui::integration::state::AppState;
use crate::interfaces::gui::integration::view_adapter;

use egui_android_framework::LifecycleObserver;

/// Компонент редактора G-Code
pub struct GCodeComponent {
    state_snapshot: AppState,
    store: StateStore<AppState>,
}

impl GCodeComponent {
    pub fn new(store: StateStore<AppState>) -> Self {
        let snapshot = store.state();
        Self {
            state_snapshot: snapshot,
            store,
        }
    }

    /// Синхронизирует снепшот с StateStore.
    /// Вызывать перед каждым render().
    pub fn sync_from_store(&mut self) {
        self.state_snapshot = self.store.state();
    }
}

impl LifecycleObserver for GCodeComponent {}

impl Component for GCodeComponent {
    type State = AppState;
    type Message = Msg;

    fn render(&self, ui: &mut egui::Ui) -> Vec<Self::Message> {
        view_adapter::view_app(&self.state_snapshot, ui.ctx())
    }

    fn handle(&mut self, _msg: Self::Message) {
        // Сообщение логируется framework. Мутация state — только в data layer.
    }

    fn state(&self) -> &Self::State {
        &self.state_snapshot
    }
}
