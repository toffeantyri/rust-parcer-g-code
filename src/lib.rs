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

    use android_activity::{
        input::{InputEvent, MotionAction},
        AndroidApp, InputStatus, MainEvent, PollEvent,
    };
    use glutin::display::GlDisplay;
    use glutin::prelude::GlSurface;
    use glutin::prelude::*;
    use raw_window_handle::HasRawWindowHandle;

    use crate::data_layer::spawn_data_layer;
    use crate::interfaces::gui::GCodeApp;

    /// Безопасно уничтожает EGL ресурсы и egui painter.
    fn destroy_egl(
        egui_painter: &mut Option<egui_glow::Painter>,
        egl_surface: &mut Option<glutin::surface::Surface<glutin::surface::WindowSurface>>,
        egl_context: &mut Option<glutin::context::PossiblyCurrentContext>,
        egl_display: &mut Option<glutin::display::Display>,
    ) {
        if let Some(ref mut p) = egui_painter {
            log::info!("destroy_egl: destroying painter");
            p.destroy();
        }
        *egui_painter = None;
        *egl_surface = None;
        *egl_context = None;
        *egl_display = None;
    }

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

        // Состояние для редактирования
        let mut last_text_change = Instant::now();
        let mut pending_text: Option<String> = None;

        // Для проброса тач-событий в egui: накапливаем события pointer
        let mut pointer_events: Vec<egui::Event> = Vec::new();
        let mut pointer_pos: Option<egui::Pos2> = None;
        let mut last_touch_down = false;

        loop {
            let mut native_window: Option<ndk::native_window::NativeWindow> = None;
            pointer_events.clear();

            app.poll_events(None, |event| match event {
                PollEvent::Wake | PollEvent::Timeout => {}
                PollEvent::Main(e) => match e {
                    MainEvent::InitWindow { .. } => {
                        let nw = app.native_window();
                        log::info!("Lifecycle: InitWindow (nw={:?})", nw.is_some());
                        needs_redraw = true;
                        if let Some(ref new_nw) = nw {
                            log::info!("InitWindow: size {}x{}", new_nw.width(), new_nw.height());
                            // Если уже был display — пересоздаём только surface для нового окна
                            if let Some(ref dpy) = egl_display {
                                log::info!("InitWindow: reusing EGL display, creating new surface");
                                // Уничтожаем старый surface (если был)
                                egl_surface = None;
                                // Создаём новый surface
                                let config = unsafe {
                                    dpy.find_configs(
                                        glutin::config::ConfigTemplateBuilder::new().build(),
                                    )
                                    .unwrap()
                                    .next()
                                    .unwrap()
                                };
                                let new_surface = unsafe {
                                    dpy.create_window_surface(
                                        &config,
                                        &glutin::surface::SurfaceAttributesBuilder::<
                                            glutin::surface::WindowSurface,
                                        >::new()
                                        .build(
                                            new_nw.raw_window_handle(),
                                            std::num::NonZeroU32::new(
                                                (new_nw.width() as u32).max(1),
                                            )
                                            .unwrap(),
                                            std::num::NonZeroU32::new(
                                                (new_nw.height() as u32).max(1),
                                            )
                                            .unwrap(),
                                        ),
                                    )
                                };
                                match new_surface {
                                    Ok(surf) => {
                                        // Перепривязываем контекст к новому surface
                                        if let Some(ref ctx) = egl_context {
                                            if let Err(e) = ctx.make_current(&surf) {
                                                log::warn!("make_current error: {:?}", e);
                                            }
                                        }
                                        egl_surface = Some(surf);
                                        log::info!("InitWindow: new EGL surface created");
                                    }
                                    Err(e) => {
                                        log::error!("create_window_surface error: {:?}", e);
                                        // Не удалось — сбросим всё и создадим заново на след. InitWindow
                                        destroy_egl(
                                            &mut egui_painter,
                                            &mut egl_surface,
                                            &mut egl_context,
                                            &mut egl_display,
                                        );
                                    }
                                }
                            }
                        }
                        native_window = nw;
                    }
                    MainEvent::Resume { .. } => {
                        log::info!("Lifecycle: Resume");
                        needs_redraw = true;
                    }
                    MainEvent::Pause { .. } => log::info!("Lifecycle: Pause"),
                    MainEvent::Stop { .. } => {
                        log::info!("Lifecycle: Stop — staying alive");
                        // Не выходим — Activity может вернуться (Resume → InitWindow).
                        // При Stop просто ждём.
                    }
                    MainEvent::Destroy { .. } => {
                        log::info!("Lifecycle: Destroy — exiting process");
                        // При Destroy завершаем процесс полностью — система загрузит native код заново.
                        // Используем exit чтобы не зависнуть на unloadNativeCode.
                        std::process::exit(0);
                    }
                    MainEvent::RedrawNeeded { .. } => {
                        needs_redraw = true;
                    }
                    _ => {}
                },
                _ => {}
            });

            // Пробрасываем тач-события в egui. Читаем InputQueue на каждом кадре.
            if egl_display.is_some() {
                if let Ok(mut iter) = app.input_events_iter() {
                    let pp = egui_ctx.pixels_per_point();
                    loop {
                        let has = iter.next(|event| match event {
                            android_activity::input::InputEvent::MotionEvent(motion) => {
                                let action = motion.action();
                                let pointer = motion.pointers().next();
                                match (action, pointer) {
                                    (android_activity::input::MotionAction::Down, Some(p))
                                    | (
                                        android_activity::input::MotionAction::PointerDown,
                                        Some(p),
                                    ) => {
                                        let pos = egui::pos2(p.x() / pp, p.y() / pp);
                                        pointer_pos = Some(pos);
                                        pointer_events.push(egui::Event::PointerButton {
                                            pos,
                                            button: egui::PointerButton::Primary,
                                            pressed: true,
                                            modifiers: egui::Modifiers::default(),
                                        });
                                        android_activity::InputStatus::Handled
                                    }
                                    (android_activity::input::MotionAction::Up, _)
                                    | (android_activity::input::MotionAction::PointerUp, _)
                                    | (android_activity::input::MotionAction::Cancel, _) => {
                                        if let Some(pos) = pointer_pos {
                                            pointer_events.push(egui::Event::PointerButton {
                                                pos,
                                                button: egui::PointerButton::Primary,
                                                pressed: false,
                                                modifiers: egui::Modifiers::default(),
                                            });
                                        }
                                        pointer_pos = None;
                                        android_activity::InputStatus::Handled
                                    }
                                    (android_activity::input::MotionAction::Move, Some(p)) => {
                                        let pos = egui::pos2(p.x() / pp, p.y() / pp);
                                        pointer_pos = Some(pos);
                                        pointer_events.push(egui::Event::PointerMoved(pos));
                                        android_activity::InputStatus::Handled
                                    }
                                    _ => android_activity::InputStatus::Unhandled,
                                }
                            }
                            _ => android_activity::InputStatus::Unhandled,
                        });
                        if !has {
                            break;
                        }
                    }
                }
            }

            // Инициализация EGL + egui при получении окна
            if egl_display.is_none() && native_window.is_some() {
                let nw = native_window.as_ref().unwrap();
                log::info!(
                    "Initializing EGL + egui... egl_display=None, nw_size={}x{}",
                    nw.width(),
                    nw.height()
                );

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

                        let nw_width = nw.width() as u32;
                        let nw_height = nw.height() as u32;
                        log::info!("Window size: {}x{}", nw_width, nw_height);

                        let surface = unsafe {
                            dpy.create_window_surface(
                                &config,
                                &glutin::surface::SurfaceAttributesBuilder::<
                                    glutin::surface::WindowSurface,
                                >::new()
                                .build(
                                    nw.raw_window_handle(),
                                    std::num::NonZeroU32::new(nw_width.max(1)).unwrap(),
                                    std::num::NonZeroU32::new(nw_height.max(1)).unwrap(),
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
                                                    Ok(mut painter) => {
                                                        // Принудительно сбрасываем все текстуры в GL —
                                                        // после пересоздания контекста старые невалидны.
                                                        unsafe {
                                                            use glow::HasContext;
                                                            let gl_ctx = painter.gl();
                                                            gl_ctx.clear_color(0.0, 0.0, 0.0, 0.0);
                                                            gl_ctx.clear(glow::COLOR_BUFFER_BIT);
                                                        }
                                                        // Перезагружаем шрифты в новом GL контексте.
                                                        egui_ctx.set_fonts(
                                                            egui::FontDefinitions::default(),
                                                        );
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

            // Всегда запрашиваем repaint пока приложение активно (для анимаций, спиннера, UI)
            needs_redraw = true;
            let now = Instant::now();
            if needs_redraw && now.duration_since(last_frame) >= Duration::from_millis(16) {
                needs_redraw = false;
                last_frame = now;

                let has_painter = egui_painter.is_some();
                let has_surface = egl_surface.is_some();
                let has_ctx = egl_context.is_some();
                if !(has_painter && has_surface && has_ctx) {
                    log::info!(
                        "Skipping frame: painter={}, surface={}, ctx={}",
                        has_painter,
                        has_surface,
                        has_ctx
                    );
                }

                if let (Some(ref mut painter), Some(ref surface), Some(ref ctx)) =
                    (&mut egui_painter, &egl_surface, &egl_context)
                {
                    let (screen_w, screen_h) = app
                        .native_window()
                        .map(|nw| (nw.width() as f32, nw.height() as f32))
                        .unwrap_or((1280.0, 720.0));
                    // Устройство 520dpi — pixels_per_point ≈ 3.25 для читаемого текста
                    egui_ctx.set_pixels_per_point(3.25);
                    // Отступы под системные панели Android (в логических точках)
                    // Отступ сверху под статус-бар (~45dp при 3.25ppp)
                    let top_inset = 45.0_f32.min(screen_h / 3.25 * 0.06);
                    // Отступ снизу под навигационную панель (Home/Back/Recent, ~48dp)
                    let bottom_inset = 48.0_f32.min(screen_h / 3.25 * 0.06);
                    let pp = egui_ctx.pixels_per_point();
                    let raw_input = egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::new(0.0, top_inset),
                            egui::vec2(screen_w / pp, (screen_h / pp) - top_inset - bottom_inset),
                        )),
                        events: pointer_events.clone(),
                        ..egui::RawInput::default()
                    };
                    let full_output = egui_ctx.run(raw_input, |ctx| {
                        use crate::interfaces::gui::view;

                        let is_busy = gcode_app.model.is_busy();

                        // 1. View → Intent: собираем намерения от UI
                        let mut all_intents =
                            view::collect_intents(ctx, is_busy, &mut gcode_app.model);
                        all_intents.extend(view::view_settings(&gcode_app.model, ctx));
                        all_intents.extend(view::view_exit_dialog(&gcode_app.model, ctx));
                        all_intents.extend(view::view_shortcuts(&gcode_app.model, ctx));
                        all_intents.extend(view::view_search_dialog(&mut gcode_app.model, ctx));
                        all_intents.extend(view::view_replace_dialog(&mut gcode_app.model, ctx));
                        all_intents.extend(view::view_axis_swap_dialog(&mut gcode_app.model, ctx));

                        // 2. Intent -> model.apply
                        for intent in &all_intents {
                            gcode_app.model.apply(intent);
                        }

                        // 3. View: статусбар и редактор
                        view::view_statusbar(&gcode_app.model, ctx);
                        view::view_editor(
                            &mut gcode_app.model,
                            ctx,
                            &gcode_app.cmd_tx,
                            &mut last_text_change,
                            &mut pending_text,
                        );

                        // Если текст изменился — очищаем подсветку ошибок
                        if pending_text.is_some() {
                            gcode_app.model.set_error_lines(Vec::new());
                        }
                    });

                    // Очищаем экран тёмно-серым фоном
                    unsafe {
                        use glow::HasContext;
                        let gl = painter.gl();
                        gl.clear_color(0.2, 0.2, 0.2, 1.0);
                        gl.clear(glow::COLOR_BUFFER_BIT);
                    }

                    // Получаем размер окна из native_window (обновляется при InitWindow)
                    let (w, h) = app
                        .native_window()
                        .map(|nw| (nw.width() as u32, nw.height() as u32))
                        .unwrap_or((1280, 720));
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
