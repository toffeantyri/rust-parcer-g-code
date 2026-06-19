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
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use android_activity::{AndroidApp, MainEvent, PollEvent};
    use glutin::display::GlDisplay;
    use glutin::prelude::GlSurface;
    use glutin::prelude::*;
    use raw_window_handle::HasRawWindowHandle;

    use crate::data_layer::spawn_data_layer;
    use crate::interfaces::gui::GCodeApp;

    #[no_mangle]
    pub fn android_main(app: AndroidApp) {
        android_logger::init_once(
            android_logger::Config::default()
                .with_tag("gcode-editor")
                .with_max_level(log::LevelFilter::Info),
        );
        log::info!("android_main: G-Code Editor Android started!");

        let (_cmd_tx, evt_rx) = spawn_data_layer();
        let mut gcode_app = GCodeApp::new(_cmd_tx, evt_rx);

        let egui_ctx = egui::Context::default();
        let mut egui_painter: Option<egui_glow::Painter> = None;
        let mut last_content_hash: u64 = 0;
        let mut needs_redraw = true;
        let mut last_frame = Instant::now();
        let mut egl_display: Option<glutin::display::Display> = None;
        let mut egl_surface: Option<glutin::surface::Surface<glutin::surface::WindowSurface>> =
            None;
        let mut egl_context: Option<glutin::context::PossiblyCurrentContext> = None;

        loop {
            let mut native_window: Option<ndk::native_window::NativeWindow> = None;

            app.poll_events(None, |event| match event {
                PollEvent::Wake | PollEvent::Timeout => {}
                PollEvent::Main(e) => match e {
                    MainEvent::InitWindow { .. } => {
                        log::info!("Lifecycle: InitWindow");
                        native_window = app.native_window();
                        needs_redraw = true;
                    }
                    MainEvent::Resume { .. } => log::info!("Lifecycle: Resume"),
                    MainEvent::Pause { .. } => log::info!("Lifecycle: Pause"),
                    MainEvent::Stop { .. } => {
                        log::info!("Lifecycle: Stop — exiting");
                        return;
                    }
                    MainEvent::Destroy { .. } => {
                        log::info!("Lifecycle: Destroy — exiting");
                        return;
                    }
                    MainEvent::RedrawNeeded { .. } => {
                        needs_redraw = true;
                    }
                    _ => {}
                },
                _ => {}
            });

            // Инициализация EGL + egui при получении окна
            if egl_display.is_none() && native_window.is_some() {
                let nw = native_window.as_ref().unwrap();
                log::info!("Initializing EGL + egui...");

                // Создаём EGL display через glutin (Android использует EGL по умолчанию)
                let display = unsafe {
                    glutin::display::Display::new(
                        raw_window_handle::RawDisplayHandle::Android(
                            raw_window_handle::AndroidDisplayHandle::empty(),
                        ),
                        glutin::display::DisplayApiPreference::Egl,
                    )
                };

                match display {
                    Ok(dpy) => {
                        let config = unsafe {
                            dpy.find_configs(glutin::config::ConfigTemplateBuilder::new().build())
                                .unwrap()
                                .next()
                                .unwrap()
                        };

                        let surface = unsafe {
                            dpy.create_window_surface(
                                &config,
                                &glutin::surface::SurfaceAttributesBuilder::<
                                    glutin::surface::WindowSurface,
                                >::new()
                                .build(
                                    nw.raw_window_handle(),
                                    std::num::NonZeroU32::new(800).unwrap(),
                                    std::num::NonZeroU32::new(600).unwrap(),
                                ),
                            )
                        };

                        match surface {
                            Ok(surface) => {
                                let ctx = unsafe {
                                    dpy.create_context(
                                        &config,
                                        &glutin::context::ContextAttributesBuilder::new()
                                            .with_context_api(glutin::context::ContextApi::Gles(
                                                None,
                                            ))
                                            .build(None),
                                    )
                                };

                                match ctx {
                                    Ok(ctx) => {
                                        let ctx = ctx.make_current(&surface);
                                        match ctx {
                                            Ok(ctx) => {
                                                let gl = unsafe {
                                                    glow::Context::from_loader_function(|name| {
                                                        let cname =
                                                            std::ffi::CStr::from_ptr(name.as_ptr()
                                                                as *const std::os::raw::c_char);
                                                        dpy.get_proc_address(cname).cast()
                                                    })
                                                };

                                                let painter = egui_glow::Painter::new(
                                                    Arc::new(gl),
                                                    "",
                                                    None,
                                                    true,
                                                );

                                                match painter {
                                                    Ok(painter) => {
                                                        egl_display = Some(dpy);
                                                        egl_surface = Some(surface);
                                                        egl_context = Some(ctx);
                                                        egui_painter = Some(painter);
                                                        log::info!("EGL + egui painter created!");
                                                        needs_redraw = true;
                                                    }
                                                    Err(e) => log::error!("Painter error: {:?}", e),
                                                }
                                            }
                                            Err(e) => {
                                                log::error!("make_current error: {:?}", e)
                                            }
                                        }
                                    }
                                    Err(e) => log::error!("create_context error: {:?}", e),
                                }
                            }
                            Err(e) => log::error!("create_window_surface error: {:?}", e),
                        }
                    }
                    Err(e) => log::error!("create display error: {:?}", e),
                }
            }

            // События от data layer
            if gcode_app.poll_events() {
                needs_redraw = true;
            }

            let new_hash = hash_str(gcode_app.model.content());
            if new_hash != last_content_hash {
                last_content_hash = new_hash;
                needs_redraw = true;
            }

            let now = Instant::now();
            if needs_redraw && now.duration_since(last_frame) >= Duration::from_millis(16) {
                needs_redraw = false;
                last_frame = now;

                if let (Some(ref mut painter), Some(ref surface), Some(ref ctx)) =
                    (&mut egui_painter, &egl_surface, &egl_context)
                {
                    let raw_input = egui::RawInput::default();
                    let full_output = egui_ctx.run(raw_input, |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.add_space(50.0);
                                ui.heading("G-Code Editor");
                                ui.add_space(20.0);
                                ui.label(format!("Status: {}", gcode_app.model.status()));
                                ui.label(format!(
                                    "File: {}",
                                    if gcode_app.model.file_path().is_empty() {
                                        "(none)"
                                    } else {
                                        gcode_app.model.file_path()
                                    }
                                ));
                            });
                        });
                    });

                    // Очищаем экран тёмно-серым фоном
                    unsafe {
                        use glow::HasContext;
                        let gl = painter.gl();
                        gl.clear_color(0.2, 0.2, 0.2, 1.0);
                        gl.clear(glow::COLOR_BUFFER_BIT);
                    }

                    let size = (800, 600); // фиксированный размер, позже сделаем динамическим
                    let (w, h) = size;
                    let primitives: Vec<egui::epaint::ClippedPrimitive> =
                        egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
                    painter.paint_and_update_textures(
                        [w, h],
                        full_output.pixels_per_point,
                        &primitives,
                        &full_output.textures_delta,
                    );

                    if let Err(e) = surface.swap_buffers(ctx) {
                        log::warn!("swap_buffers error: {:?}", e);
                    }
                }
            }

            // Небольшая задержка чтобы не грузить CPU
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn hash_str(s: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::hash::DefaultHasher::new();
        s.hash(&mut h);
        h.finish()
    }
}

#[cfg(target_os = "android")]
pub use android::android_main;
