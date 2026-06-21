//! View для Android — полноценный UI с тулбаром, редактором, диалогами.
//! Отличается от desktop-версии:
//! - Нет меню-бара (все действия через кнопки тулбара)
//! - Нет горячих клавиш (кроме системных)
//! - Кнопки крупнее для сенсорного ввода

use std::sync::mpsc;
use std::time::Instant;

use egui::TextStyle;

use crate::data_layer::EditorCommand;

use crate::shared::i18n;

use crate::infrastructure::highlight::build_highlighted_job;

use crate::interfaces::gui::intent::{AxisSwapMode, Intent};
use crate::interfaces::gui::model::Model;

/// Собирает намерения от UI: Drawer (боковое меню) для Android.
/// На телефоне узкий экран — всё управление через бургер-меню слева.
pub fn collect_intents(ctx: &egui::Context, is_busy: bool, model: &mut Model) -> Vec<Intent> {
    let mut intents = Vec::new();

    // Верхняя панель — только бургер и имя файла (минимально)
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // Кнопка бургер — используем id для хранения состояния между кадрами
            let burger_id = ui.id().with("burger");
            let btn = egui::Button::new("☰").min_size(egui::vec2(40.0, 40.0));
            let resp = ui.add(btn);
            if resp.clicked() {
                // Меняем drawer_open немедленно, чтобы SidePanel показалась на этом же кадре
                let new_state = !model.flag_drawer_open();
                model.set_drawer_open(new_state);
            }

            // Имя файла (если есть)
            let path = model.file_path();
            if !path.is_empty() {
                let display = if path.len() > 25 {
                    format!("...{}", &path[path.len().saturating_sub(22)..])
                } else {
                    path.to_string()
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

    // Drawer — боковая панель, читаем актуальное состояние из модели
    if model.flag_drawer_open() {
        // Затемнение фона при открытом дровере.
        let screen_rect = ctx.screen_rect();
        let overlay_rect = egui::Rect::from_two_pos(
            screen_rect.left_top() + egui::vec2(260.0, 0.0),
            screen_rect.right_bottom(),
        );
        // Рисуем затемнение поверх всего
        egui::Area::new("drawer_overlay".into())
            .fixed_pos(egui::Pos2::ZERO)
            .order(egui::Order::Foreground)
            .interactable(true)
            .show(ctx, |ui| {
                let resp = ui.allocate_rect(overlay_rect, egui::Sense::click());
                ui.painter()
                    .rect_filled(overlay_rect, 0.0, egui::Color32::from_black_alpha(100));
                if resp.clicked() || resp.hovered() {
                    // Если overlay зарегил клик или просто наведение — значит тач был вне дровера.
                    // clicked() + проверка что overlay hovered = клик вне дровера.
                }
            });
        // Если был тач вне панели дровера — закрываем.
        // Проверяем что тач был правее 260px (ширина SidePanel).
        // Используем pressed для немедленной реакции.
        if ctx.input(|i| i.pointer.any_pressed()) {
            let pressed_pos = ctx.input(|i| i.pointer.interact_pos());
            if let Some(pos) = pressed_pos {
                if pos.x > 260.0 {
                    model.set_drawer_open(false);
                    ctx.request_repaint();
                }
            }
        }

        egui::SidePanel::left("drawer")
            .resizable(false)
            .default_width(260.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.set_min_width(240.0);
                    ui.add_space(16.0);

                    // Заголовок
                    ui.label(egui::RichText::new("G-Code Editor").size(18.0).strong());
                    ui.separator();
                    ui.add_space(8.0);

                    // Файл
                    drawer_section(ui, &i18n::locale().menu.file, |ui| {
                        drawer_item(
                            ui,
                            &i18n::locale().menu.open,
                            !is_busy,
                            &mut intents,
                            Intent::OpenFile,
                        );
                        drawer_item(
                            ui,
                            &i18n::locale().menu.save,
                            !is_busy,
                            &mut intents,
                            Intent::SaveFile,
                        );
                        drawer_item(
                            ui,
                            &i18n::locale().menu.save_as,
                            !is_busy,
                            &mut intents,
                            Intent::SaveAs,
                        );
                        drawer_item_always(
                            ui,
                            &i18n::locale().menu.close,
                            &mut intents,
                            Intent::CloseFile,
                        );
                    });

                    // Правка
                    drawer_section(ui, &i18n::locale().menu.edit, |ui| {
                        drawer_item(
                            ui,
                            &i18n::locale().menu.format,
                            !is_busy,
                            &mut intents,
                            Intent::Format,
                        );
                        drawer_item(
                            ui,
                            &i18n::locale().menu.validate,
                            !is_busy,
                            &mut intents,
                            Intent::Validate,
                        );
                    });

                    // Инструменты
                    drawer_section(ui, "Инструменты", |ui| {
                        drawer_item_always(
                            ui,
                            &i18n::locale().menu.search,
                            &mut intents,
                            Intent::ToggleSearch,
                        );
                        drawer_item_always(
                            ui,
                            &i18n::locale().menu.replace,
                            &mut intents,
                            Intent::ToggleReplace,
                        );
                        drawer_item_always(
                            ui,
                            &i18n::locale().menu.axis_swap,
                            &mut intents,
                            Intent::ToggleAxisSwap,
                        );
                    });

                    // Настройки
                    drawer_section(ui, &i18n::locale().menu.settings, |ui| {
                        drawer_item_always(
                            ui,
                            &i18n::locale().menu.format_settings,
                            &mut intents,
                            Intent::ToggleSettings,
                        );
                    });

                    ui.separator();

                    // Язык
                    let is_ru = model.format_settings().language == "ru";
                    let is_en = model.format_settings().language == "en";
                    if ui
                        .add(
                            egui::Button::new(&i18n::locale().menu.lang_ru)
                                .min_size(egui::vec2(220.0, 40.0))
                                .selected(is_ru),
                        )
                        .clicked()
                        && !is_ru
                    {
                        intents.push(Intent::SetLanguage("ru".to_string()));
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
                        intents.push(Intent::SetLanguage("en".to_string()));
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // О программе и выход
                    drawer_item_always(
                        ui,
                        &i18n::locale().menu.shortcuts,
                        &mut intents,
                        Intent::ToggleShortcuts,
                    );
                    ui.add_space(16.0);
                    if ui
                        .add(
                            egui::Button::new(&i18n::locale().menu.exit)
                                .min_size(egui::vec2(220.0, 44.0)),
                        )
                        .clicked()
                    {
                        intents.push(Intent::Exit);
                    }
                });
            });
    }

    intents
}

/// Отрисовывает секцию в Drawer с заголовком-разделителем.
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

/// Пункт Drawer (enabled).
fn drawer_item(
    ui: &mut egui::Ui,
    label: &str,
    enabled: bool,
    intents: &mut Vec<Intent>,
    intent: Intent,
) {
    if ui
        .add_enabled(
            enabled,
            egui::Button::new(label).min_size(egui::vec2(220.0, 40.0)),
        )
        .clicked()
    {
        intents.push(intent);
    }
}

/// Пункт Drawer (всегда включён).
fn drawer_item_always(ui: &mut egui::Ui, label: &str, intents: &mut Vec<Intent>, intent: Intent) {
    if ui
        .add(egui::Button::new(label).min_size(egui::vec2(220.0, 40.0)))
        .clicked()
    {
        intents.push(intent);
    }
}

/// Отрисовывает строку состояния.
pub fn view_statusbar(model: &Model, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let mut status = model.status().to_string();
            if model.is_busy() {
                let dots = ((ctx.input(|i| i.time) * 4.0) as usize) % 4;
                status.push(' ');
                for i in 0..3 {
                    if i < dots {
                        status.push('.');
                    } else {
                        status.push(' ');
                    }
                }
            }
            ui.label(egui::RichText::new(&status).size(12.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new("v0.1")
                        .size(11.0)
                        .color(egui::Color32::GRAY),
                );
            });
        });
    });
}

/// Отрисовывает окно настроек форматирования.
pub fn view_settings(model: &Model, ctx: &egui::Context) -> Vec<Intent> {
    let mut intents = Vec::new();
    let mut open_copy = model.settings_open();

    if !open_copy {
        return intents;
    }

    egui::Window::new(&i18n::locale().settings.title)
        .open(&mut open_copy)
        .resizable(false)
        .default_size([320.0, 220.0])
        .show(ctx, |ui| {
            ui.label(&i18n::locale().settings.renumber_step);
            ui.horizontal(|ui| {
                for &step in &[1u32, 10, 100] {
                    let selected = model.format_settings().renumber_step == step;
                    let btn =
                        egui::Button::new(format!("{}", step)).min_size(egui::vec2(48.0, 36.0));
                    if ui.add(btn.selected(selected)).clicked() {
                        intents.push(Intent::SetRenumberStep(step));
                    }
                }
            });
            ui.add_space(8.0);
            ui.label(&i18n::locale().settings.examples);
            let step = model.format_settings().renumber_step;
            ui.label(format!("N{} N{} N{} ...", step, step * 2, step * 3));
            ui.add_space(12.0);
            let mut skip = model.format_settings().skip_empty_lines;
            if ui
                .checkbox(&mut skip, &i18n::locale().settings.skip_empty)
                .changed()
            {
                intents.push(Intent::SetSkipEmptyLines(skip));
            }
        });

    if !open_copy {
        intents.push(Intent::ToggleSettings);
    }

    intents
}

/// Отрисовывает редактор кода и отслеживает изменения.
pub fn view_editor(
    model: &mut Model,
    ctx: &egui::Context,
    _cmd_tx: &mpsc::Sender<EditorCommand>,
    last_text_change: &mut Instant,
    pending_text: &mut Option<String>,
) {
    let content_before = model.content().to_string();
    let mut edit_content = model.content().to_string();

    // Вычисляем зону подсветки поиска/замены (start..end) в байтах.
    let search_highlight = if model.replace_open() && !model.replace_matches().is_empty() {
        let idx = model.replace_index();
        let byte_pos = model.replace_matches().get(idx).copied().unwrap_or(0);
        let len = model.replace_find().len();
        Some((byte_pos, byte_pos + len))
    } else if !model.search_matches().is_empty() {
        let idx = model.search_index();
        let byte_pos = model.search_matches().get(idx).copied().unwrap_or(0);
        let len = model.search_query().len();
        Some((byte_pos, byte_pos + len))
    } else {
        None
    };

    egui::CentralPanel::default().show(ctx, |ui| {
        let editor_id = ui.next_auto_id();

        if model.editor_needs_focus() {
            ctx.memory_mut(|mem| mem.request_focus(editor_id));
            model.set_editor_needs_focus(false);
        }

        egui::ScrollArea::vertical()
            .id_salt("editor_scroll")
            .show(ui, |ui| {
                ui.add_sized(
                    ui.available_size(),
                    egui::TextEdit::multiline(&mut edit_content)
                        .id(editor_id)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .desired_rows(50)
                        .font(TextStyle::Monospace)
                        .layouter(&mut |_ui: &egui::Ui, text: &str, _wrap_width: f32| {
                            let job =
                                build_highlighted_job(text, model.error_lines(), search_highlight);
                            _ui.fonts(|f| f.layout_job(job))
                        }),
                );
            });
    });

    if edit_content != content_before {
        model.set_content(edit_content);
        model.set_modified(true);
        *pending_text = Some(model.content().to_string());
        *last_text_change = Instant::now();
    }
}

/// Отрисовывает диалог подтверждения выхода/закрытия.
pub fn view_exit_dialog(model: &Model, ctx: &egui::Context) -> Vec<Intent> {
    let mut intents = Vec::new();
    if !model.show_exit_dialog() {
        return intents;
    }

    // На Android Escape недоступен через клавиатуру — только через системную кнопку Back
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        intents.push(Intent::CancelAction);
        return intents;
    }

    let mut is_open = true;
    egui::Window::new(&i18n::locale().dialog.exit_title)
        .open(&mut is_open)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([350.0, 140.0])
        .show(ctx, |ui| {
            ui.label(&i18n::locale().dialog.confirm_save);
            ui.add_space(12.0);

            let labels = [
                &i18n::locale().dialog.btn_save,
                &i18n::locale().dialog.btn_discard,
                &i18n::locale().dialog.btn_cancel,
            ];
            let actions = [
                Intent::ConfirmSave,
                Intent::DiscardAndContinue,
                Intent::CancelAction,
            ];

            ui.horizontal(|ui| {
                for (i, label) in labels.iter().enumerate() {
                    let btn = egui::Button::new(*label).min_size(egui::vec2(100.0, 40.0));
                    if ui.add(btn).clicked() {
                        intents.push(actions[i].clone());
                    }
                }
            });
        });

    if !is_open && model.show_exit_dialog() {
        intents.push(Intent::CancelAction);
    }

    intents
}

/// Отрисовывает окно горячих клавиш.
pub fn view_shortcuts(model: &Model, ctx: &egui::Context) -> Vec<Intent> {
    let mut intents = Vec::new();
    if !model.shortcuts_open() {
        return intents;
    }

    let mut open_copy = true;
    egui::Window::new("⌨ ".to_string() + &i18n::locale().menu.shortcuts)
        .open(&mut open_copy)
        .resizable(false)
        .default_size([420.0, 360.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            use egui::RichText;

            let shortcuts = [
                ("Открыть", &i18n::locale().menu.open),
                ("Сохранить", &i18n::locale().menu.save),
                ("Сохранить как", &i18n::locale().menu.save_as),
                ("Форматировать (F5)", &i18n::locale().menu.format),
                ("Проверить (F6)", &i18n::locale().menu.validate),
                ("Поиск", &i18n::locale().menu.search),
                ("Замена", &i18n::locale().menu.replace),
            ];

            ui.label(
                RichText::new(&i18n::locale().menu.shortcuts_title)
                    .size(16.0)
                    .strong(),
            );
            ui.separator();
            ui.add_space(8.0);

            for (key, desc) in &shortcuts {
                ui.horizontal(|ui| {
                    ui.monospace(*key);
                    ui.label("  —  ");
                    ui.label(*desc);
                });
            }
        });

    if !open_copy {
        intents.push(Intent::ToggleShortcuts);
    }

    intents
}

/// Отрисовывает диалог поиска.
pub fn view_search_dialog(model: &mut Model, ctx: &egui::Context) -> Vec<Intent> {
    let mut intents = Vec::new();
    if !model.search_open() {
        return intents;
    }

    let mut open_copy = true;
    let loc = i18n::locale();
    egui::Window::new(&loc.search.title)
        .open(&mut open_copy)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([360.0, 150.0])
        .show(ctx, |ui| {
            let mut query = model.search_query().to_string();
            let search_field_id = ui.id().with("search_input");
            if model.search_focus_needed() {
                ctx.memory_mut(|mem| mem.request_focus(search_field_id));
            }
            let resp = ui.add_sized(
                [ui.available_width(), 32.0],
                egui::TextEdit::singleline(&mut query)
                    .id(search_field_id)
                    .hint_text(&loc.search.search_hint)
                    .font(TextStyle::Monospace),
            );
            if resp.has_focus() && model.search_focus_needed() {
                model.set_search_focus_needed(false);
            }
            if query != model.search_query() {
                intents.push(Intent::SetSearchQuery(query));
            }
            if resp.lost_focus() && ui.ctx().input(|i| i.key_pressed(egui::Key::Enter)) {
                intents.push(Intent::FindNext);
                model.set_search_focus_needed(true);
            }
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                intents.push(Intent::CloseSearchDialog);
                return;
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui
                    .add(egui::Button::new(&loc.search.btn_find).min_size(egui::vec2(64.0, 36.0)))
                    .clicked()
                {
                    intents.push(Intent::FindNext);
                }
                if ui
                    .add(egui::Button::new(&loc.search.btn_cancel).min_size(egui::vec2(64.0, 36.0)))
                    .clicked()
                {
                    intents.push(Intent::CloseSearchDialog);
                }
                let found = model.search_matches().len();
                if !model.search_last_query().is_empty() {
                    if found > 0 {
                        ui.label(format!("{} / {}", model.search_index() + 1, found));
                    } else {
                        ui.label(&loc.search.not_found);
                    }
                }
            });
        });

    if !open_copy {
        intents.push(Intent::CloseSearchDialog);
    }

    intents
}

/// Отрисовывает диалог замены.
pub fn view_replace_dialog(model: &mut Model, ctx: &egui::Context) -> Vec<Intent> {
    let mut intents = Vec::new();
    if !model.replace_open() {
        return intents;
    }

    let mut open_copy = true;
    let loc = i18n::locale();
    egui::Window::new(&loc.replace.title)
        .open(&mut open_copy)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([380.0, 220.0])
        .show(ctx, |ui| {
            // Escape → закрыть
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                intents.push(Intent::CloseSearchDialog);
                return;
            }

            // Поле «Найти»
            let mut find = model.replace_find().to_string();
            ui.label(&loc.replace.find_hint);
            let find_field_id = ui.id().with("replace_find_input");
            if model.replace_focus_needed() {
                ctx.memory_mut(|mem| mem.request_focus(find_field_id));
            }
            let _find_resp = ui.add_sized(
                [ui.available_width(), 32.0],
                egui::TextEdit::singleline(&mut find)
                    .id(find_field_id)
                    .hint_text(&loc.replace.find_hint)
                    .font(TextStyle::Monospace),
            );
            if _find_resp.has_focus() && model.replace_focus_needed() {
                model.set_replace_focus_needed(false);
            }
            if find != model.replace_find() {
                intents.push(Intent::SetReplaceFind(find.clone()));
            }
            if _find_resp.lost_focus()
                && !find.is_empty()
                && ui.ctx().input(|i| i.key_pressed(egui::Key::Enter))
            {
                intents.push(Intent::ReplaceFindNext);
                model.set_replace_focus_needed(true);
            }

            ui.add_space(4.0);

            // Поле «Заменить на»
            let mut replace_with = model.replace_with().to_string();
            ui.label(&loc.replace.replace_hint);
            let _replace_resp = ui.add_sized(
                [ui.available_width(), 32.0],
                egui::TextEdit::singleline(&mut replace_with)
                    .hint_text(&loc.replace.replace_hint)
                    .font(TextStyle::Monospace),
            );
            if replace_with != model.replace_with() {
                intents.push(Intent::SetReplaceWith(replace_with));
            }

            ui.add_space(8.0);

            // Кнопки
            let can_find = !model.replace_find().is_empty();
            let can_replace =
                can_find && !model.replace_with().is_empty() && !model.replace_matches().is_empty();
            let can_replace_all = can_find && !model.replace_with().is_empty();
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        can_find,
                        egui::Button::new(&loc.replace.btn_find).min_size(egui::vec2(64.0, 36.0)),
                    )
                    .clicked()
                {
                    intents.push(Intent::ReplaceFindNext);
                }
                if ui
                    .add_enabled(
                        can_replace,
                        egui::Button::new(&loc.replace.btn_replace)
                            .min_size(egui::vec2(64.0, 36.0)),
                    )
                    .clicked()
                {
                    intents.push(Intent::ReplaceOne);
                }
                if ui
                    .add_enabled(
                        can_replace_all,
                        egui::Button::new(&loc.replace.btn_replace_all)
                            .min_size(egui::vec2(80.0, 36.0)),
                    )
                    .clicked()
                {
                    intents.push(Intent::ReplaceAll);
                }
                if ui
                    .add(
                        egui::Button::new(&loc.replace.btn_cancel).min_size(egui::vec2(64.0, 36.0)),
                    )
                    .clicked()
                {
                    intents.push(Intent::CloseSearchDialog);
                }
            });

            // Статус
            if !model.replace_last_find().is_empty() {
                ui.add_space(4.0);
                let found = model.replace_matches().len();
                if found > 0 {
                    ui.label(format!("{} / {}", model.replace_index() + 1, found));
                } else {
                    ui.label(&loc.replace.not_found);
                }
            }
        });

    if !open_copy {
        intents.push(Intent::CloseSearchDialog);
    }

    intents
}

/// Отрисовывает диалог замены осей.
pub fn view_axis_swap_dialog(model: &mut Model, ctx: &egui::Context) -> Vec<Intent> {
    let mut intents = Vec::new();
    if !model.axis_swap_open() {
        return intents;
    }

    let mut open_copy = true;
    let loc = i18n::locale();
    egui::Window::new(&loc.axis_swap.title)
        .open(&mut open_copy)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([340.0, 240.0])
        .show(ctx, |ui| {
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                intents.push(Intent::ToggleAxisSwap);
                return;
            }

            // Режимы
            let is_swap = *model.axis_swap_mode() == AxisSwapMode::Swap;
            let is_invert = *model.axis_swap_mode() == AxisSwapMode::Invert;

            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(&loc.axis_swap.mode_swap)
                            .min_size(egui::vec2(80.0, 36.0))
                            .selected(is_swap),
                    )
                    .clicked()
                {
                    intents.push(Intent::SetAxisSwapMode(AxisSwapMode::Swap));
                }
                if ui
                    .add(
                        egui::Button::new(&loc.axis_swap.mode_invert)
                            .min_size(egui::vec2(80.0, 36.0))
                            .selected(is_invert),
                    )
                    .clicked()
                {
                    intents.push(Intent::SetAxisSwapMode(AxisSwapMode::Invert));
                }
            });

            ui.add_space(8.0);

            if is_swap {
                let mut axis1 = model.axis_swap_axis1().to_string();
                let mut axis2 = model.axis_swap_axis2().to_string();

                ui.horizontal(|ui| {
                    ui.label(&loc.axis_swap.axis1_hint);
                    ui.add_sized(
                        [60.0, 32.0],
                        egui::TextEdit::singleline(&mut axis1)
                            .char_limit(1)
                            .hint_text("Z")
                            .font(TextStyle::Monospace),
                    );
                });
                if axis1 != model.axis_swap_axis1() {
                    intents.push(Intent::SetSwapAxis1(axis1.to_uppercase()));
                }

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label(&loc.axis_swap.axis2_hint);
                    ui.add_sized(
                        [60.0, 32.0],
                        egui::TextEdit::singleline(&mut axis2)
                            .char_limit(1)
                            .hint_text("X")
                            .font(TextStyle::Monospace),
                    );
                });
                if axis2 != model.axis_swap_axis2() {
                    intents.push(Intent::SetSwapAxis2(axis2.to_uppercase()));
                }
            } else {
                let mut invert_axis = model.axis_invert_axis().to_string();
                ui.horizontal(|ui| {
                    ui.label(&loc.axis_swap.axis1_hint);
                    ui.add_sized(
                        [60.0, 32.0],
                        egui::TextEdit::singleline(&mut invert_axis)
                            .char_limit(1)
                            .hint_text("X")
                            .font(TextStyle::Monospace),
                    );
                });
                if invert_axis != model.axis_invert_axis() {
                    intents.push(Intent::SetInvertAxis(invert_axis.to_uppercase()));
                }
            }

            ui.add_space(12.0);

            let can_apply = if is_swap {
                model.axis_swap_axis1().len() == 1 && model.axis_swap_axis2().len() == 1
            } else {
                model.axis_invert_axis().len() == 1
            };
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        can_apply,
                        egui::Button::new(&loc.axis_swap.btn_apply)
                            .min_size(egui::vec2(80.0, 36.0)),
                    )
                    .clicked()
                {
                    intents.push(Intent::ApplyAxisSwap);
                }
                if ui
                    .add(
                        egui::Button::new(&loc.axis_swap.btn_cancel)
                            .min_size(egui::vec2(80.0, 36.0)),
                    )
                    .clicked()
                {
                    intents.push(Intent::ToggleAxisSwap);
                }
            });
        });

    if !open_copy {
        intents.push(Intent::ToggleAxisSwap);
    }

    intents
}
