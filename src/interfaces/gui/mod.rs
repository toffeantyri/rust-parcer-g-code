//! Слой GUI: архитектура MVI (Model-View-Intent)

mod app;
mod intent;
mod model;
mod update;
mod view;

pub use app::GCodeApp;
pub use intent::Intent;
pub use model::{FormatSettings, Model};
pub use view::{
    collect_intents, view_editor, view_exit_dialog, view_settings, view_shortcuts, view_statusbar,
};
