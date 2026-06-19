//! Платформозависимые реализации (файловый ввод-вывод).
//!
//! Содержит трейт `FilePicker` и реализации для десктопа и Android.

#[cfg(not(target_os = "android"))]
#[path = "desktop.rs"]
mod platform_impl;

#[cfg(target_os = "android")]
#[path = "android.rs"]
mod platform_impl;

pub use platform_impl::{pick_file, save_file};
