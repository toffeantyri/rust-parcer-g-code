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
/// ANativeActivity_onCreate генерируется android-activity (feature native-activity).
#[cfg(target_os = "android")]
mod android {
    use std::sync::mpsc;
    use std::time::Duration;

    use android_activity::{AndroidApp, MainEvent, PollEvent};

    use crate::data_layer::{spawn_data_layer, EditorCommand};
    use crate::interfaces::gui::GCodeApp;

    /// Состояние Android-приложения.
    struct AndroidState {
        gcode_app: GCodeApp,
        #[allow(dead_code)]
        cmd_tx: mpsc::Sender<EditorCommand>,
        last_content_hash: u64,
        needs_redraw: bool,
    }

    /// Простой хеш для отслеживания изменений контента.
    fn hash_str(s: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::hash::DefaultHasher::new();
        s.hash(&mut h);
        h.finish()
    }

    #[no_mangle]
    pub fn android_main(app: AndroidApp) {
        android_logger::init_once(
            android_logger::Config::default()
                .with_tag("gcode-editor")
                .with_max_level(log::LevelFilter::Info),
        );
        log::info!("android_main: G-Code Editor Android started!");

        let (cmd_tx, evt_rx) = spawn_data_layer();
        let gcode_app = GCodeApp::new(cmd_tx.clone(), evt_rx);
        let content_hash = hash_str(gcode_app.model.content());

        let mut state = AndroidState {
            gcode_app,
            cmd_tx,
            last_content_hash: content_hash,
            needs_redraw: true,
        };

        loop {
            // Обрабатываем события с дедлайном 16ms (~60 fps max)
            app.poll_events(Some(Duration::from_millis(16)), |event| match event {
                PollEvent::Wake | PollEvent::Timeout => {}
                PollEvent::Main(e) => match e {
                    MainEvent::InitWindow { .. } => {
                        log::info!("Lifecycle: InitWindow");
                        state.needs_redraw = true;
                    }
                    MainEvent::Resume { .. } => log::info!("Lifecycle: Resume"),
                    MainEvent::Pause { .. } => log::info!("Lifecycle: Pause"),
                    MainEvent::Stop { .. } => log::info!("Lifecycle: Stop"),
                    MainEvent::Start { .. } => log::info!("Lifecycle: Start"),
                    MainEvent::Destroy { .. } => {
                        log::info!("Lifecycle: Destroy — exiting");
                        return;
                    }
                    MainEvent::RedrawNeeded { .. } => {
                        state.needs_redraw = true;
                    }
                    MainEvent::WindowResized { .. } => {
                        state.needs_redraw = true;
                    }
                    _ => {}
                },
                _ => {}
            });

            // Проверяем события от data layer (GCodeApp сам дренирует канал)
            if state.gcode_app.poll_events() {
                state.needs_redraw = true;
            }

            // Проверяем изменился ли контент с момента прошлого кадра
            let new_hash = hash_str(state.gcode_app.model.content());
            if new_hash != state.last_content_hash {
                state.last_content_hash = new_hash;
                state.needs_redraw = true;
            }

            // Рендеринг только при необходимости
            if state.needs_redraw {
                state.needs_redraw = false;
                // TODO: render_ui(&mut state);
                log::debug!("Redraw frame");
            }
        }
    }
}

#[cfg(target_os = "android")]
pub use android::android_main;
