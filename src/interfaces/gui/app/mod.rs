//! Слой App — точка входа, связывает UI с data layer.
//! desktop.rs — десктоп (eframe), android.rs — Android (android-activity).

#[cfg(not(target_os = "android"))]
#[path = "desktop.rs"]
#[allow(clippy::module_inception)]
mod app;

#[cfg(target_os = "android")]
#[path = "android.rs"]
#[allow(clippy::module_inception)]
mod app;

#[cfg(all(test, feature = "desktop"))]
#[path = "desktop_tests.rs"]
mod tests;

pub use app::GCodeApp;
