//! Парсер и форматтер G-кода
//!
//! Архитектура: Clean Architecture
//! - domain: сущности (AST, Token)
//! - application: use cases (парсер, форматтер)
//! - infrastructure: внешние зависимости (лексер, ввод-вывод)
//! - interfaces: CLI/API обработчики
//! - shared: общие типы (ошибки, конфиги)

pub mod application;
pub mod data_layer;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;
pub mod shared;
