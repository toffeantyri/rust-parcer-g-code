//! Парсер и форматтер G-кода
//!
//! Архитектура: Clean Architecture
//! - domain: сущности (AST, Token)
//! - application: use cases (парсер, форматтер)
//! - infrastructure: внешние зависимости (лексер, ввод-вывод)
//! - interfaces: CLI/API обработчики
//! - shared: общие типы (ошибки, конфиги)

pub mod application;
pub mod data_layer;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;
pub mod shared;

/// Точка входа Android через native-activity.
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: android_activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("gcode-editor")
            .with_max_level(log::LevelFilter::Info),
    );
    log::info!("android_main: G-Code Editor Android started!");

    loop {
        app.poll_events(None, |event| {
            use android_activity::PollEvent;
            match event {
                PollEvent::Wake => {
                    log::debug!("Lifecycle: Wake");
                }
                PollEvent::Timeout => {
                    log::trace!("Lifecycle: Timeout");
                }
                PollEvent::Main(main_event) => {
                    use android_activity::MainEvent;
                    match main_event {
                        MainEvent::InitWindow { .. } => {
                            log::info!("Lifecycle: InitWindow");
                        }
                        MainEvent::TerminateWindow { .. } => {
                            log::info!("Lifecycle: TerminateWindow");
                        }
                        MainEvent::WindowResized { .. } => {
                            log::info!("Lifecycle: WindowResized");
                        }
                        MainEvent::RedrawNeeded { .. } => {
                            log::debug!("Lifecycle: RedrawNeeded");
                        }
                        MainEvent::ContentRectChanged { .. } => {
                            log::info!("Lifecycle: ContentRectChanged");
                        }
                        MainEvent::Resume { .. } => {
                            log::info!("Lifecycle: Resume");
                        }
                        MainEvent::Pause { .. } => {
                            log::info!("Lifecycle: Pause");
                        }
                        MainEvent::Stop { .. } => {
                            log::info!("Lifecycle: Stop");
                        }
                        MainEvent::Destroy { .. } => {
                            log::info!("Lifecycle: Destroy — exiting");
                            return;
                        }
                        MainEvent::Start { .. } => {
                            log::info!("Lifecycle: Start");
                        }
                        MainEvent::ConfigChanged { .. } => {
                            log::info!("Lifecycle: ConfigChanged");
                        }
                        MainEvent::LowMemory { .. } => {
                            log::warn!("Lifecycle: LowMemory");
                        }
                        _ => {
                            log::debug!("Lifecycle: Other event");
                        }
                    }
                }
                _ => {
                    log::trace!("Lifecycle: Unknown poll event");
                }
            }
        });
    }
}
