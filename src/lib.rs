//! Парсер и форматтер G-кода
//!
//! Архитектура: Clean Architecture
//! - domain: сущности (AST, Token)
//! - application: use cases (парсер, форматтер)
//! - infrastructure: внешние зависимости (лексер, ввод-вывод)
//! - interfaces: CLI/API обработчики + GUI (egui MVI)
//! - data_layer: отдельный поток (pipeline)
//! - shared: общие типы (ошибки, конфиги)

pub mod application;
pub mod data_layer;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;
pub mod shared;

/// Точка входа Android через egui-android-framework.
#[cfg(target_os = "android")]
#[no_mangle]
pub fn android_main(app: android_activity::AndroidApp) {
    use crate::interfaces::gui::integration::app::GCodeApp;
    use egui_android_framework::android::run;
    run::<GCodeApp>(app);
}
