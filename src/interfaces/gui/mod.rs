//! Слой GUI: архитектура MVI (Model-View-Intent)
//!
//! Структура папок по MVI-слоям:
//! - `model/` — состояние приложения
//! - `intent/` — намерения пользователя (enum)
//! - `update/` — редьюсер: мутирует model на основе intent
//! - `view/` — отрисовка UI, возвращает intents
//! - `app/` — точка входа eframe, соединяет слои

pub use app::GCodeApp;
pub use intent::Intent;
pub use model::{FormatSettings, Model};
pub use view::{
    collect_intents, view_editor, view_exit_dialog, view_replace_dialog, view_search_dialog,
    view_settings, view_shortcuts, view_statusbar,
};

#[cfg(any(target_os = "android", not(feature = "desktop")))]
pub(crate) mod integration;

pub(crate) mod app;
pub(crate) mod intent;
pub(crate) mod model;
pub(crate) mod update;
pub(crate) mod view;
