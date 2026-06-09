//! Model — состояние редактора G-кода

/// Состояние приложения
#[derive(Default)]
pub struct Model {
    /// Содержимое редактора G-кода
    pub content: String,
    /// Путь к текущему файлу
    pub file_path: String,
    /// Текст в строке состояния
    pub status: String,
}
