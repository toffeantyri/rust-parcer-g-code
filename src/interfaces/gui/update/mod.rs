//! Слой Update — редьюсер: применяет намерения к модели.
//! Содержит Model::apply() и сохранение/загрузку настроек.

#[cfg(test)]
#[path = "update_tests.rs"]
mod tests;
#[allow(clippy::module_inception)]
mod update;

/// Реэкспорт для data_layer (интеграция Android).
/// Функции лежат в update.rs — это часть слоя interfaces,
/// а data_layer тоже часть interfaces, поэтому импорт разрешён.
pub use update::{invert_axes_by_letter, swap_axes};
