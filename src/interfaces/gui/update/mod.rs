//! Слой Update — редьюсер: применяет намерения к модели.
//! Содержит Model::apply() и сохранение/загрузку настроек.

#[cfg(test)]
#[path = "update_tests.rs"]
mod tests;
#[allow(clippy::module_inception)]
mod update;
