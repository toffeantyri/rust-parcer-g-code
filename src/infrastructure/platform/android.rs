//! Файловый пикер для Android (SAF).
//! Будет реализован при сборке под Android.

/// Заглушка — на Android будет использовать SAF.
pub fn pick_file() -> Option<std::path::PathBuf> {
    None
}

pub fn save_file() -> Option<std::path::PathBuf> {
    None
}
