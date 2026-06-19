//! Слой View — отрисовка UI, возвращает намерения.
//! Содержит меню, тулбар, редактор, статусбар, диалоги.

#[allow(clippy::module_inception)]
mod view;

pub use view::{
    collect_intents, view_editor, view_exit_dialog, view_replace_dialog, view_search_dialog,
    view_settings, view_shortcuts, view_statusbar,
};
