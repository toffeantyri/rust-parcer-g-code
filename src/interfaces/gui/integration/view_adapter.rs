//! View Adapter — чистая функция отрисовки под egui-android-framework.
//!
//! Полный аналог view_android.rs (collect_intents + view_editor + диалоги).
//! Отличия:
//! - принимает &AppState (только чтение), возвращает Vec<Msg>
//! - не мутирует state (drawer_open и др. флаги идут через Msg::ToggleDrawer)
//! - не знает о mpsc, data layer, egui-контексте (кроме ctx для отрисовки)

use egui::TextStyle;

use crate::infrastructure::highlight::build_highlighted_job;
use crate::interfaces::gui::integration::msg::Msg;
use crate::interfaces::gui::integration::state::AppState;
use crate::interfaces::gui::intent::AxisSwapMode;
use crate::shared::i18n;

// Буфер для прямого редактирования текста в TextEdit.
// Единственное исключение из MVI — TextEdit мутируется напрямую.
thread_local! {
    static TEXT_BUFFER: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

/// Главная точка входа View — отрисовывает всё приложение.
pub fn view_app(state: &AppState, ctx: &egui::Context) -> Vec<Msg> {
    let mut msgs = Vec::new();

    view_top_bar(state, ctx, &mut msgs);

    if state.drawer_open {
        view_drawer_overlay(state, ctx, &mut msgs);
        view_drawer(state, ctx, &mut msgs);
    }

    view_editor(state, ctx, &mut msgs);
    view_statusbar(state, ctx);

    if state.settings_open {
        view_settings_dialog(state, ctx, &mut msgs);
    }
    if state.search_open {
        view_search_dialog(state, ctx, &mut msgs);
    }
    if state.replace_open {
        view_replace_dialog(state, ctx, &mut msgs);
    }
    if state.axis_swap_open {
        view_axis_swap_dialog(state, ctx, &mut msgs);
    }
    if state.shortcuts_open {
        view_shortcuts_dialog(state, ctx, &mut msgs);
    }
    if state.show_exit_dialog {
        view_exit_dialog(state, ctx, &mut msgs);
    }

    msgs
}

// ── Верхняя панель ──

fn view_top_bar(state: &AppState, ctx: &egui::Context, msgs: &mut Vec<Msg>) {
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let btn = egui::Button::new("☰").min_size(egui::vec2(40.0, 40.0));
            if ui.add(btn).clicked() {
                msgs.push(Msg::ToggleDrawer);
            }

            let path = &state.file_path;
            if !path.is_empty() {
                let display = if path.len() > 25 {
                    format!("...{}", &path[path.len().saturating_sub(22)..])
                } else {
                    path.clone()
                };
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(&display)
                            .size(14.0)
                            .color(egui::Color32::GRAY)
                            .weak(),
                    );
                });
            }
        });
    });
}

// ── Затемнение фона Drawer ──

fn view_drawer_overlay(state: &AppState, ctx: &egui::Context, msgs: &mut Vec<Msg>) {
    let screen_rect = ctx.screen_rect();
    let overlay_rect = egui::Rect::from_two_pos(
        screen_rect.left_top() + egui::vec2(260.0, 0.0),
        screen_rect.right_bottom(),
    );

    egui::Area::new("drawer_overlay".into())
        .fixed_pos(egui::Pos2::ZERO)
        .order(egui::Order::Foreground)
        .interactable(true)
        .show(ctx, |ui| {
            ui.painter()
                .rect_filled(overlay_rect, 0.0, egui::Color32::from_black_alpha(100));
        });
}

// ── Боковое меню Drawer ──

fn view_drawer(state: &AppState, ctx: &egui::Context, msgs: &mut Vec<Msg>) {
    let is_busy = state.is_busy;

    egui::SidePanel::left("drawer")
        .resizable(false)
        .default_width(260.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .id_source("drawer_scroll")
                .drag_to_scroll(false)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .show(ui, |ui| {
                    ui.set_min_width(240.0);
                    ui.add_space(16.0);

                    ui.label(egui::RichText::new("G-Code Editor").size(18.0).strong());
                    ui.separator();
                    ui.add_space(8.0);

                    drawer_section(ui, &i18n::locale().menu.file, |ui| {
                        drawer_item(ui, &i18n::locale().menu.open, !is_busy, msgs, Msg::OpenFile);
                        drawer_item(ui, &i18n::locale().menu.save, !is_busy, msgs, Msg::SaveFile);
                        drawer_item(
                            ui,
                            &i18n::locale().menu.save_as,
                            !is_busy,
                            msgs,
                            Msg::SaveAs,
                        );
                        drawer_item_always(ui, &i18n::locale().menu.close, msgs, Msg::CloseFile);
                    });

                    drawer_section(ui, &i18n::locale().menu.edit, |ui| {
                        drawer_item(ui, &i18n::locale().menu.format, !is_busy, msgs, Msg::Format);
                        drawer_item(
                            ui,
                            &i18n::locale().menu.validate,
                            !is_busy,
                            msgs,
                            Msg::Validate,
                        );
                        drawer_item_always(
                            ui,
                            &i18n::locale().menu.search,
                            msgs,
                            Msg::ToggleSearch,
                        );
                        drawer_item_always(
                            ui,
                            &i18n::locale().menu.replace,
                            msgs,
                            Msg::ToggleReplace,
                        );
                        drawer_item_always(
                            ui,
                            &i18n::locale().menu.axis_swap,
                            msgs,
                            Msg::ToggleAxisSwap,
                        );
                    });

                    drawer_section(ui, &i18n::locale().menu.settings, |ui| {
                        drawer_item_always(
                            ui,
                            &i18n::locale().menu.format_settings,
                            msgs,
                            Msg::ToggleSettings,
                        );
                    });

                    ui.separator();

                    let is_ru = state.format_settings.language == "ru";
                    let is_en = state.format_settings.language == "en";
                    if ui
                        .add(
                            egui::Button::new(&i18n::locale().menu.lang_ru)
                                .min_size(egui::vec2(220.0, 40.0))
                                .selected(is_ru),
                        )
                        .clicked()
                        && !is_ru
                    {
                        msgs.push(Msg::SetLanguage("ru".to_string()));
                    }
                    if ui
                        .add(
                            egui::Button::new(&i18n::locale().menu.lang_en)
                                .min_size(egui::vec2(220.0, 40.0))
                                .selected(is_en),
                        )
                        .clicked()
                        && !is_en
                    {
                        msgs.push(Msg::SetLanguage("en".to_string()));
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    drawer_item_always(
                        ui,
                        &i18n::locale().menu.shortcuts,
                        msgs,
                        Msg::ToggleShortcuts,
                    );
                    ui.add_space(16.0);
                    if ui
                        .add(
                            egui::Button::new(&i18n::locale().menu.exit)
                                .min_size(egui::vec2(220.0, 44.0)),
                        )
                        .clicked()
                    {
                        msgs.push(Msg::Exit);
                    }
                });
        });

    // Тач вне Drawer (правее 260px) — закрываем
    if ctx.input(|i| i.pointer.any_pressed()) {
        if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
            if pos.x > 260.0 {
                msgs.push(Msg::ToggleDrawer);
            }
        }
    }
}

fn drawer_section(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    ui.label(
        egui::RichText::new(title)
            .size(13.0)
            .color(egui::Color32::GRAY)
            .strong(),
    );
    ui.add_space(2.0);
    add_contents(ui);
    ui.add_space(8.0);
}

fn drawer_item(ui: &mut egui::Ui, label: &str, enabled: bool, msgs: &mut Vec<Msg>, msg: Msg) {
    if ui
        .add_enabled(
            enabled,
            egui::Button::new(label).min_size(egui::vec2(220.0, 40.0)),
        )
        .clicked()
    {
        msgs.push(msg);
    }
}

fn drawer_item_always(ui: &mut egui::Ui, label: &str, msgs: &mut Vec<Msg>, msg: Msg) {
    if ui
        .add(egui::Button::new(label).min_size(egui::vec2(220.0, 40.0)))
        .clicked()
    {
        msgs.push(msg);
    }
}

// ── Редактор ──

fn view_editor(state: &AppState, ctx: &egui::Context, msgs: &mut Vec<Msg>) {
    egui::CentralPanel::default().show(ctx, |ui| {
        let available = ui.available_size();
        let text_edit_height = available.y - 4.0;

        let font_id = TextStyle::Body.resolve(ui.style());
        let job = build_highlighted_job(state.content.as_str(), &state.error_lines, None);

        let (_, painter) = ui.allocate_painter(
            egui::vec2(available.x, text_edit_height),
            egui::Sense::hover(),
        );

        let top_pos = ui.next_widget_position();
        let text_pos = egui::pos2(top_pos.x + 4.0, top_pos.y + 4.0);
        let galley = ui.fonts(|f| f.layout_job(job));
        painter.add(egui::epaint::TextShape::new(
            text_pos,
            galley,
            egui::Color32::WHITE,
        ));

        // TextEdit — исключение: мутируем thread_local буфер
        TEXT_BUFFER.with(|buf| {
            let mut buffer = buf.borrow_mut();
            if buffer.is_empty() && !state.content.is_empty() {
                *buffer = state.content.clone();
            }

            let resp = egui::TextEdit::multiline(&mut *buffer)
                .id(ui.id().with("code_editor"))
                .font(TextStyle::Body)
                .desired_width(available.x)
                .desired_rows((text_edit_height / 20.0) as usize)
                .show(ui);

            if resp.response.changed() {
                msgs.push(Msg::TextChanged(buffer.clone()));
            }
        });
    });
}

// ── Строка состояния ──

fn view_statusbar(state: &AppState, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let mut status = state.status.clone();
            if state.is_busy {
                status = format!("⏳ {}", status);
            }
            if !state.file_path.is_empty() {
                let save_indicator = if state.modified { " ●" } else { "" };
                status = format!("{}{}", status, save_indicator);
            }
            ui.label(
                egui::RichText::new(&status)
                    .size(12.0)
                    .color(egui::Color32::GRAY),
            );
        });
    });
}

// ── Диалог настроек ──

fn view_settings_dialog(state: &AppState, ctx: &egui::Context, msgs: &mut Vec<Msg>) {
    egui::Window::new(&i18n::locale().settings.title)
        .id("settings_window".into())
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&i18n::locale().settings.renumber_step);
            });

            let mut step = state.format_settings.renumber_step as i32;
            if ui
                .add(egui::Slider::new(&mut step, 0..=1000).text(""))
                .changed()
            {
                msgs.push(Msg::SetRenumberStep(step.max(0) as u32));
            }

            let mut skip = state.format_settings.skip_empty_lines;
            if ui
                .checkbox(&mut skip, &i18n::locale().settings.skip_empty)
                .changed()
            {
                msgs.push(Msg::SetSkipEmptyLines(skip));
            }

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                let is_ru = state.format_settings.language == "ru";
                let is_en = state.format_settings.language == "en";
                if ui
                    .selectable_label(is_ru, &i18n::locale().menu.lang_ru)
                    .clicked()
                    && !is_ru
                {
                    msgs.push(Msg::SetLanguage("ru".to_string()));
                }
                if ui
                    .selectable_label(is_en, &i18n::locale().menu.lang_en)
                    .clicked()
                    && !is_en
                {
                    msgs.push(Msg::SetLanguage("en".to_string()));
                }
            });

            ui.add_space(8.0);
            if ui.button(&i18n::locale().dialog.btn_cancel).clicked() {
                msgs.push(Msg::ToggleSettings);
            }
        });
}

// ── Диалог поиска ──

fn view_search_dialog(state: &AppState, ctx: &egui::Context, msgs: &mut Vec<Msg>) {
    let mut search_query = state.search_query.clone();

    egui::Window::new(&i18n::locale().search.title)
        .id("search_window".into())
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut search_query)
                        .hint_text(&i18n::locale().search.search_hint)
                        .desired_width(200.0),
                );
                if resp.changed() {
                    msgs.push(Msg::SetSearchQuery(search_query.clone()));
                }
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    msgs.push(Msg::DoSearch);
                }
                if state.search_focus_needed {
                    resp.request_focus();
                }

                if ui.button(&i18n::locale().search.btn_find).clicked() {
                    msgs.push(Msg::DoSearch);
                }
            });

            if !state.search_matches.is_empty() {
                ui.label(format!(
                    "{} / {}",
                    state.search_index + 1,
                    state.search_matches.len()
                ));
                ui.horizontal(|ui| {
                    if ui.button("◀").clicked() {
                        msgs.push(Msg::FindNext);
                    }
                    if ui.button("▶").clicked() {
                        msgs.push(Msg::FindNext);
                    }
                });
            }

            if ui.button(&i18n::locale().search.btn_cancel).clicked() {
                msgs.push(Msg::CloseSearchDialog);
            }
        });
}

// ── Диалог замены ──

fn view_replace_dialog(state: &AppState, ctx: &egui::Context, msgs: &mut Vec<Msg>) {
    let mut find_text = state.replace_find.clone();
    let mut replace_text = state.replace_with.clone();

    egui::Window::new(&i18n::locale().replace.title)
        .id("replace_window".into())
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut find_text)
                        .hint_text(&i18n::locale().replace.find_hint)
                        .desired_width(200.0),
                );
                if resp.changed() {
                    msgs.push(Msg::SetReplaceFind(find_text.clone()));
                }
                if state.replace_focus_needed {
                    resp.request_focus();
                }
                if ui.button(&i18n::locale().replace.btn_find).clicked() {
                    msgs.push(Msg::DoReplaceSearch);
                }
            });

            ui.horizontal(|ui| {
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut replace_text)
                        .hint_text(&i18n::locale().replace.replace_hint)
                        .desired_width(200.0),
                );
                if resp.changed() {
                    msgs.push(Msg::SetReplaceWith(replace_text.clone()));
                }
            });

            if !state.replace_matches.is_empty() {
                ui.label(format!(
                    "{} / {}",
                    state.replace_index + 1,
                    state.replace_matches.len()
                ));
                ui.horizontal(|ui| {
                    if ui.button(&i18n::locale().replace.btn_replace).clicked() {
                        msgs.push(Msg::ReplaceOne);
                    }
                    if ui.button(&i18n::locale().replace.btn_replace_all).clicked() {
                        msgs.push(Msg::ReplaceAll);
                    }
                    if ui.button("▶").clicked() {
                        msgs.push(Msg::ReplaceFindNext);
                    }
                });
            }

            if ui.button(&i18n::locale().replace.btn_cancel).clicked() {
                msgs.push(Msg::CloseSearchDialog);
            }
        });
}

// ── Диалог замены осей ──

fn view_axis_swap_dialog(state: &AppState, ctx: &egui::Context, msgs: &mut Vec<Msg>) {
    let mut axis1 = state.axis_swap_axis1.clone();
    let mut axis2 = state.axis_swap_axis2.clone();
    let mut invert_axis = state.axis_invert_axis.clone();

    egui::Window::new(&i18n::locale().axis_swap.title)
        .id("axis_swap_window".into())
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&i18n::locale().axis_swap.title);
                let is_swap = state.axis_swap_mode == AxisSwapMode::Swap;
                let is_invert = state.axis_swap_mode == AxisSwapMode::Invert;
                if ui
                    .selectable_label(is_swap, &i18n::locale().axis_swap.mode_swap)
                    .clicked()
                    && !is_swap
                {
                    msgs.push(Msg::SetAxisSwapMode(AxisSwapMode::Swap));
                }
                if ui
                    .selectable_label(is_invert, &i18n::locale().axis_swap.mode_invert)
                    .clicked()
                    && !is_invert
                {
                    msgs.push(Msg::SetAxisSwapMode(AxisSwapMode::Invert));
                }
            });

            ui.add_space(8.0);

            match state.axis_swap_mode {
                AxisSwapMode::Swap => {
                    ui.horizontal(|ui| {
                        ui.label(&i18n::locale().axis_swap.axis1_hint);
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut axis1)
                                .desired_width(60.0)
                                .char_limit(1),
                        );
                        if resp.changed() {
                            msgs.push(Msg::SetSwapAxis1(axis1.to_uppercase()));
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(&i18n::locale().axis_swap.axis2_hint);
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut axis2)
                                .desired_width(60.0)
                                .char_limit(1),
                        );
                        if resp.changed() {
                            msgs.push(Msg::SetSwapAxis2(axis2.to_uppercase()));
                        }
                    });
                }
                AxisSwapMode::Invert => {
                    ui.horizontal(|ui| {
                        ui.label(&i18n::locale().axis_swap.axis1_hint);
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut invert_axis)
                                .desired_width(60.0)
                                .char_limit(1),
                        );
                        if resp.changed() {
                            msgs.push(Msg::SetInvertAxis(invert_axis.to_uppercase()));
                        }
                    });
                }
            }

            ui.add_space(8.0);
            if ui
                .add_enabled(
                    !state.is_busy,
                    egui::Button::new(&i18n::locale().axis_swap.btn_apply),
                )
                .clicked()
            {
                msgs.push(Msg::ApplyAxisSwap);
            }

            if ui.button(&i18n::locale().axis_swap.btn_cancel).clicked() {
                msgs.push(Msg::ToggleAxisSwap);
            }
        });
}

// ── Диалог шорткатов ──

fn view_shortcuts_dialog(state: &AppState, ctx: &egui::Context, msgs: &mut Vec<Msg>) {
    egui::Window::new(&i18n::locale().menu.shortcuts)
        .id("shortcuts_window".into())
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            let shortcuts = [
                ("F5", &i18n::locale().menu.format),
                ("F6", &i18n::locale().menu.validate),
                ("Ctrl+O", &i18n::locale().menu.open),
                ("Ctrl+S", &i18n::locale().menu.save),
                ("Ctrl+Shift+S", &i18n::locale().menu.save_as),
                ("Ctrl+F", &i18n::locale().menu.search),
                ("Ctrl+H", &i18n::locale().menu.replace),
            ];
            for (key, desc) in shortcuts {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(key).strong());
                    ui.label(desc);
                });
            }

            if ui.button(&i18n::locale().dialog.btn_cancel).clicked() {
                msgs.push(Msg::ToggleShortcuts);
            }
        });
}

// ── Диалог выхода ──

fn view_exit_dialog(state: &AppState, ctx: &egui::Context, msgs: &mut Vec<Msg>) {
    egui::Window::new(&i18n::locale().dialog.exit_title)
        .id("exit_dialog".into())
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(&i18n::locale().dialog.confirm_save);
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button(&i18n::locale().dialog.btn_save).clicked() {
                    msgs.push(Msg::ConfirmSave);
                }
                if ui.button(&i18n::locale().dialog.btn_discard).clicked() {
                    msgs.push(Msg::DiscardAndContinue);
                }
                if ui.button(&i18n::locale().dialog.btn_cancel).clicked() {
                    msgs.push(Msg::CancelAction);
                }
            });
        });
}
