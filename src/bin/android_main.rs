//! Минимальная точка входа Android для проверки что APK собирается и запускается.
//! Не использует code_parser (чтобы обойти rfd через eframe).

#[cfg(not(target_os = "android"))]
fn main() {
    eprintln!("Только для Android.");
}

#[cfg(target_os = "android")]
mod android_impl {
    use android_activity::AndroidApp;

    #[no_mangle]
    pub fn android_main(app: AndroidApp) {
        android_logger::init_once(
            android_logger::Config::default()
                .with_tag("gcode-editor")
                .with_max_level(log::LevelFilter::Info),
        );

        log::info!("G-Code Editor Android Hello World!");

        loop {
            app.poll_events(None, |event| match event {
                android_activity::PollEvent::Wake => {}
                android_activity::PollEvent::Timeout => {}
                android_activity::PollEvent::Main(main_event) => {
                    log::info!("Event: {:?}", main_event);
                }
                _ => {}
            });
        }
    }
}

#[cfg(target_os = "android")]
pub use android_impl::android_main;
