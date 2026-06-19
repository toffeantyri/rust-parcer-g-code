//! Слой View — отрисовка UI, возвращает намерения.
//! Содержит меню, тулбар, редактор, статусбар, диалоги.

#[cfg(not(target_os = "android"))]
#[path = "view_desktop.rs"]
#[allow(clippy::module_inception)]
mod view;

#[cfg(target_os = "android")]
#[path = "view_android.rs"]
#[allow(clippy::module_inception)]
mod view;

pub use view::{
    collect_intents, view_axis_swap_dialog, view_editor, view_exit_dialog, view_replace_dialog,
    view_search_dialog, view_settings, view_shortcuts, view_statusbar,
};
