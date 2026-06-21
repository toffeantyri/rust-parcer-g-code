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

pub use view::*;
