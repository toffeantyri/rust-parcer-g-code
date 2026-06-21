//! Слой App — точка входа, связывает UI с data layer.
//! desktop.rs — десктоп (eframe), android.rs — Android (android-activity).

#[cfg(feature = "desktop")]
#[path = "desktop.rs"]
#[allow(clippy::module_inception)]
mod app;

#[cfg(any(target_os = "android", not(feature = "desktop")))]
#[path = "android.rs"]
#[allow(clippy::module_inception)]
mod app;

#[cfg(all(test, feature = "desktop"))]
#[path = "desktop_tests.rs"]
mod tests;

pub use app::GCodeApp;
