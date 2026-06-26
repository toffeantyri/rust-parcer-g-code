//! Integration — адаптация G-Code Editor под egui-android-framework.
//!
//! Содержит все слои для работы с фреймворком:
//! - `state.rs` — AppState (аналог Model, Clone + Send + Sync)
//! - `msg.rs` — Msg (сообщения между View и Data Layer)
//! - `view_adapter.rs` — чистая функция отрисовки UI
//! - `component.rs` — Component (обёртка над StateStore + View)
//! - `data_layer.rs` — Data Layer в фоновом потоке
//! - `app.rs` — Application (точка входа)
//!
//! Используется ТОЛЬКО для Android (feature = "android").
//! Desktop остаётся на старой архитектуре (eframe).

pub mod app;
pub mod component;
pub mod data_layer;
pub mod msg;
pub mod state;
pub mod view_adapter;
