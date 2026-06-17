//! Слой App — точка входа eframe, связывает UI с data layer.
//! Содержит GCodeApp — главную структуру приложения.

mod app;
#[cfg(test)]
#[path = "app_tests.rs"]
mod tests;

pub use app::GCodeApp;
