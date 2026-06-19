//! Файловый пикер для десктопа (rfd).

use rfd::FileDialog;

/// Реализация FilePicker через rfd.
pub fn pick_file() -> Option<std::path::PathBuf> {
    FileDialog::new().pick_file()
}

/// Сохранить файл через rfd.
pub fn save_file() -> Option<std::path::PathBuf> {
    FileDialog::new().save_file()
}
