//! Слой View — отрисовка UI, возвращает намерения.
//! Содержит меню, тулбар, редактор, статусбар, диалоги.

#[cfg(feature = "desktop")]
#[path = "view_desktop.rs"]
#[allow(clippy::module_inception)]
mod view;

#[cfg(not(feature = "desktop"))]
#[path = "view_android.rs"]
#[allow(clippy::module_inception)]
mod view;

pub use view::*;

#[cfg(test)]
#[path = "view_android_tests.rs"]
mod tests;
