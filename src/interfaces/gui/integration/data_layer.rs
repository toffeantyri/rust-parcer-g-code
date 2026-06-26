//! Data Layer для egui-android-framework.
//!
//! Запускается в отдельном потоке, получает Msg через mpsc,
//! меняет состояние через store.update(), уведомляет UI через notify_tx.

use std::sync::mpsc;

use egui_android_framework::store::StateStore;

use crate::data_layer::pipeline;
use crate::interfaces::gui::integration::msg::Msg;
use crate::interfaces::gui::integration::state::{AppState, PendingActionUi};
use crate::interfaces::gui::intent::AxisSwapMode;
use crate::shared::i18n;

/// Запускает data layer в фоновом потоке.
pub fn spawn_data_layer(
    cmd_rx: mpsc::Receiver<Msg>,
    store: StateStore<AppState>,
    notify_tx: mpsc::Sender<()>,
) {
    std::thread::spawn(move || {
        data_loop(cmd_rx, store, notify_tx);
    });
}

fn data_loop(
    cmd_rx: mpsc::Receiver<Msg>,
    store: StateStore<AppState>,
    notify_tx: mpsc::Sender<()>,
) {
    loop {
        match cmd_rx.recv() {
            Ok(msg) => {
                handle_msg(msg, &store, &notify_tx);
            }
            Err(_) => break,
        }
    }
}

fn handle_msg(msg: Msg, store: &StateStore<AppState>, notify_tx: &mpsc::Sender<()>) {
    match msg {
        // ── Изменения состояния без пайплайна ──
        Msg::ToggleSettings => {
            store.update(|s| s.settings_open = !s.settings_open);
            notify(notify_tx);
        }
        Msg::SetRenumberStep(step) => {
            store.update(|s| s.format_settings.renumber_step = step);
            notify(notify_tx);
        }
        Msg::SetSkipEmptyLines(skip) => {
            store.update(|s| s.format_settings.skip_empty_lines = skip);
            notify(notify_tx);
        }
        Msg::ToggleShortcuts => {
            store.update(|s| s.shortcuts_open = !s.shortcuts_open);
            notify(notify_tx);
        }
        Msg::ToggleSearch => {
            store.update(|s| {
                s.search_open = !s.search_open;
                if s.search_open {
                    s.search_focus_needed = true;
                }
            });
            notify(notify_tx);
        }
        Msg::ToggleReplace => {
            store.update(|s| {
                s.replace_open = !s.replace_open;
                if s.replace_open {
                    s.replace_focus_needed = true;
                }
            });
            notify(notify_tx);
        }
        Msg::CloseSearchDialog => {
            store.update(|s| {
                s.search_open = false;
                s.replace_open = false;
            });
            notify(notify_tx);
        }
        Msg::ToggleAxisSwap => {
            store.update(|s| s.axis_swap_open = !s.axis_swap_open);
            notify(notify_tx);
        }
        Msg::ToggleDrawer => {
            store.update(|s| s.drawer_open = !s.drawer_open);
            notify(notify_tx);
        }
        Msg::SetSearchQuery(q) => {
            store.update(|s| s.search_query = q);
            notify(notify_tx);
        }
        Msg::SetReplaceFind(f) => {
            store.update(|s| s.replace_find = f);
            notify(notify_tx);
        }
        Msg::SetReplaceWith(w) => {
            store.update(|s| s.replace_with = w);
            notify(notify_tx);
        }
        Msg::SetSwapAxis1(a) => {
            store.update(|s| s.axis_swap_axis1 = a);
            notify(notify_tx);
        }
        Msg::SetSwapAxis2(a) => {
            store.update(|s| s.axis_swap_axis2 = a);
            notify(notify_tx);
        }
        Msg::SetInvertAxis(a) => {
            store.update(|s| s.axis_invert_axis = a);
            notify(notify_tx);
        }
        Msg::SetAxisSwapMode(m) => {
            store.update(|s| s.axis_swap_mode = m);
            notify(notify_tx);
        }
        Msg::SetLanguage(lang) => {
            store.update(|s| {
                s.format_settings.language = lang.clone();
            });
            i18n::set_lang(&lang);
            store.update(|s| {
                s.status = "Language changed".to_string();
            });
            notify(notify_tx);
        }
        Msg::TextChanged(content) => {
            store.update(|s| s.content = content);
            notify(notify_tx);
        }
        Msg::CancelAction => {
            store.update(|s| {
                s.show_exit_dialog = false;
                s.pending_action = None;
            });
            notify(notify_tx);
        }

        // ── Диалог выхода ──
        Msg::ConfirmSave => {
            let st = store.state();
            let pending = st.pending_action.clone();
            store.update(|s| {
                s.show_exit_dialog = false;
                s.pending_action = None;
                match &pending {
                    Some(PendingActionUi::Exit) => {}
                    Some(PendingActionUi::CloseFile) => {
                        s.content.clear();
                        s.file_path.clear();
                        s.modified = false;
                        s.status = i18n::locale().status.file_closed.clone();
                    }
                    _ => {}
                }
            });
            notify(notify_tx);
        }
        Msg::DiscardAndContinue => {
            let st = store.state();
            let pending = st.pending_action.clone();
            store.update(|s| {
                s.show_exit_dialog = false;
                s.pending_action = None;
                match pending {
                    Some(PendingActionUi::Exit) => {}
                    Some(PendingActionUi::CloseFile) => {
                        s.content.clear();
                        s.file_path.clear();
                        s.modified = false;
                        s.status = i18n::locale().status.file_closed.clone();
                    }
                    _ => {}
                }
            });
            notify(notify_tx);
        }

        // ── Поиск ──
        Msg::DoSearch => {
            let st = store.state();
            let content = st.content.clone();
            let query = st.search_query.clone();
            let prev_matches = st.search_matches.clone();
            let prev_index = st.search_index;
            let prev_last_query = st.search_last_query.clone();
            let matches = find_all_occurrences(&content, &query);
            let start_index = if query == prev_last_query && !matches.is_empty() {
                (prev_index + 1) % matches.len()
            } else {
                0
            };
            store.update(|s| {
                s.search_matches = matches.clone();
                s.search_index = if matches.is_empty() { 0 } else { start_index };
                s.search_last_query = query.clone();
                s.status = if matches.is_empty() {
                    "Not found".to_string()
                } else {
                    format!("Found: {}", matches.len())
                };
            });
            notify(notify_tx);
        }
        Msg::FindNext => {
            store.update(|s| {
                if !s.search_matches.is_empty() {
                    s.search_index = (s.search_index + 1) % s.search_matches.len();
                    s.status = format!("{}/{}", s.search_index + 1, s.search_matches.len());
                }
            });
            notify(notify_tx);
        }
        Msg::DoReplaceSearch => {
            let st = store.state();
            let content = st.content.clone();
            let query = st.replace_find.clone();
            let matches = find_all_occurrences(&content, &query);
            store.update(|s| {
                s.replace_matches = matches.clone();
                s.replace_index = 0;
                s.replace_last_find = query.clone();
                s.status = if matches.is_empty() {
                    "Not found".to_string()
                } else {
                    format!("Found: {}", matches.len())
                };
            });
            notify(notify_tx);
        }
        Msg::ReplaceFindNext => {
            store.update(|s| {
                if !s.replace_matches.is_empty() {
                    s.replace_index = (s.replace_index + 1) % s.replace_matches.len();
                }
            });
            notify(notify_tx);
        }
        Msg::ReplaceOne => {
            let st = store.state();
            let content = st.content.clone();
            let find = st.replace_find.clone();
            let replace_with = st.replace_with.clone();
            let index = st.replace_index;
            let matches = st.replace_matches.clone();
            if index < matches.len() {
                let pos = matches[index];
                let mut new_content = content;
                if let Some(end) = new_content[pos..].find(&find) {
                    new_content.replace_range(pos..pos + end + find.len(), &replace_with);
                    let shift = replace_with.len() as isize - find.len() as isize;
                    let new_matches: Vec<usize> = matches
                        .iter()
                        .enumerate()
                        .map(|(i, m)| {
                            if i > index {
                                (*m as isize + shift) as usize
                            } else {
                                *m
                            }
                        })
                        .collect();
                    store.update(|s| {
                        s.content = new_content;
                        s.replace_matches = new_matches;
                        s.modified = true;
                    });
                    notify(notify_tx);
                }
            }
        }
        Msg::ReplaceAll => {
            let st = store.state();
            let content = st.content.clone();
            let find = st.replace_find.clone();
            let replace_with = st.replace_with.clone();
            let new_content = content.replace(&find, &replace_with);
            store.update(|s| {
                s.content = new_content;
                s.replace_matches.clear();
                s.replace_index = 0;
                s.modified = true;
                s.status = "All replaced".to_string();
            });
            notify(notify_tx);
        }

        // ── Замена осей ──
        Msg::ApplyAxisSwap => {
            let st = store.state();
            let content = st.content.clone();
            let mode = st.axis_swap_mode.clone();
            let axis1 = st.axis_swap_axis1.clone();
            let axis2 = st.axis_swap_axis2.clone();
            let invert_axis = st.axis_invert_axis.clone();
            let new_content = match mode {
                AxisSwapMode::Swap => {
                    crate::interfaces::gui::update::swap_axes(&content, &axis1, &axis2)
                }
                AxisSwapMode::Invert => {
                    crate::interfaces::gui::update::invert_axes_by_letter(&content, &invert_axis)
                }
            };
            store.update(|s| {
                s.content = new_content;
                s.modified = true;
                s.status = "Axes swapped".to_string();
            });
            notify(notify_tx);
        }

        // ── Пайплайн ──
        Msg::Format => {
            store.update(|s| {
                s.is_busy = true;
                s.status = i18n::locale().status.formatting.clone();
            });
            notify(notify_tx);
            let st = store.state();
            let content = st.content.clone();
            let renumber_step = st.format_settings.renumber_step;
            let skip_empty_lines = st.format_settings.skip_empty_lines;
            let result = pipeline::format_code(&content, renumber_step, skip_empty_lines);
            match result {
                Ok((formatted, errors)) => {
                    store.update(|s| {
                        if !formatted.is_empty() {
                            s.content = formatted;
                            s.modified = true;
                        }
                        s.error_lines = errors.iter().map(|e| e.line).filter(|l| *l > 0).collect();
                        s.is_busy = false;
                        s.status = if errors.is_empty() {
                            i18n::locale().status.formatted.clone()
                        } else {
                            let first = &errors[0];
                            i18n::fmt_errors_found(errors.len(), &first.message, first.line)
                        };
                    });
                    notify(notify_tx);
                }
                Err(e) => {
                    store.update(|s| {
                        s.is_busy = false;
                        s.status = i18n::fmt_error(&e.to_string());
                    });
                    notify(notify_tx);
                }
            }
        }
        Msg::Validate => {
            store.update(|s| {
                s.is_busy = true;
                s.status = i18n::locale().status.validating.clone();
            });
            notify(notify_tx);
            let content = store.state().content.clone();
            let result = pipeline::validate_code(&content);
            match result {
                Ok(errors) => {
                    store.update(|s| {
                        s.error_lines = errors.iter().map(|e| e.line).filter(|l| *l > 0).collect();
                        s.is_busy = false;
                        s.status = if errors.is_empty() {
                            i18n::locale().status.no_errors.clone()
                        } else {
                            let first = &errors[0];
                            i18n::fmt_errors_found(errors.len(), &first.message, first.line)
                        };
                    });
                    notify(notify_tx);
                }
                Err(e) => {
                    store.update(|s| {
                        s.is_busy = false;
                        s.status = i18n::fmt_error(&e.to_string());
                    });
                    notify(notify_tx);
                }
            }
        }

        // ── Файлы ──
        Msg::OpenFile => {
            store.update(|s| {
                s.is_busy = true;
                s.status = "Opening...".to_string();
            });
            notify(notify_tx);
        }
        Msg::SaveFile | Msg::SaveAs => {
            store.update(|s| {
                s.is_busy = true;
                s.status = "Saving...".to_string();
            });
            notify(notify_tx);
        }
        Msg::CloseFile => {
            let st = store.state();
            let modified = st.modified;
            let path = st.file_path.clone();
            if modified && !path.is_empty() {
                store.update(|s| {
                    s.show_exit_dialog = true;
                    s.pending_action = Some(PendingActionUi::CloseFile);
                });
            } else {
                store.update(|s| {
                    s.content.clear();
                    s.file_path.clear();
                    s.modified = false;
                    s.status = i18n::locale().status.file_closed.clone();
                });
            }
            notify(notify_tx);
        }
        Msg::Exit => {
            let st = store.state();
            let modified = st.modified;
            let path = st.file_path.clone();
            if modified && !path.is_empty() {
                store.update(|s| {
                    s.show_exit_dialog = true;
                    s.pending_action = Some(PendingActionUi::Exit);
                });
                notify(notify_tx);
            } else {
                return;
            }
        }

        Msg::Formatted { .. }
        | Msg::Validated { .. }
        | Msg::FileLoaded { .. }
        | Msg::FileSaved { .. }
        | Msg::Notify { .. }
        | Msg::Idle => {}
    }
}

fn notify(tx: &mpsc::Sender<()>) {
    let _ = tx.send(());
}

fn find_all_occurrences(text: &str, query: &str) -> Vec<usize> {
    if query.is_empty() {
        return Vec::new();
    }
    let lower_text = text.to_lowercase();
    let lower_query = query.to_lowercase();
    let mut positions = Vec::new();
    let mut start = 0;
    while let Some(pos) = lower_text[start..].find(&lower_query) {
        positions.push(start + pos);
        start += pos + 1;
    }
    positions
}
