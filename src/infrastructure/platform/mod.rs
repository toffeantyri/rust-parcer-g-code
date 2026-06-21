//! Платформозависимые реализации (файловый ввод-вывод).
//!
//! Содержит трейт `FilePicker` и реализации для десктопа и Android.

#[cfg(feature = "desktop")]
#[path = "desktop.rs"]
mod platform_impl;

#[cfg(not(feature = "desktop"))]
#[path = "android.rs"]
mod platform_impl;

pub use platform_impl::{pick_file, save_file};
