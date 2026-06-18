//! Слой Model — состояние редактора G-кода

#[allow(clippy::module_inception)]
mod model;
#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;

pub use model::{FormatSettings, Model, PendingAction};
