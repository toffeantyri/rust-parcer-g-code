//! Тесты data layer

use super::*;
use crate::data_layer::{
    DialogCommand, DialogEvent, FileCommand, FileEvent, PipelineCommand, PipelineEvent,
};

// -----------------------------------------------------------------------
// Pipeline: команды форматирования / валидации через каналы
// -----------------------------------------------------------------------

#[test]
fn test_pipeline_format_via_channel() {
    let (tx, rx) = spawn_data_layer();
    tx.send(EditorCommand::Pipeline(PipelineCommand::Format {
        content: "G0 X10".to_string(),
        renumber_step: 0,
        skip_empty_lines: true,
    }))
    .unwrap();

    // Ждём: NotifyUser + Pipeline(Formatted) + Idle
    let events: Vec<EditorEvent> = rx.iter().take(3).collect();
    assert_eq!(events.len(), 3);

    assert!(
        matches!(
            &events[0],
            EditorEvent::Dialog(DialogEvent::NotifyUser { .. })
        ),
        "первое событие — NotifyUser"
    );
    match &events[1] {
        EditorEvent::Pipeline(PipelineEvent::Formatted { content, errors }) => {
            assert!(content.contains("G0"));
            assert!(errors.is_empty());
        }
        _ => panic!("второе событие — Formatted, но получено {:?}", events[1]),
    }
    assert!(
        matches!(&events[2], EditorEvent::Dialog(DialogEvent::Idle)),
        "третье событие — Idle"
    );
}

#[test]
fn test_pipeline_validate_via_channel() {
    let (tx, rx) = spawn_data_layer();
    tx.send(EditorCommand::Pipeline(PipelineCommand::Validate(
        "G0 X10 Y20".to_string(),
    )))
    .unwrap();

    let events: Vec<EditorEvent> = rx.iter().take(3).collect();
    assert_eq!(events.len(), 3);

    assert!(
        matches!(
            &events[0],
            EditorEvent::Dialog(DialogEvent::NotifyUser { .. })
        ),
        "первое событие — NotifyUser"
    );
    match &events[1] {
        EditorEvent::Pipeline(PipelineEvent::Validated { errors }) => {
            assert!(errors.is_empty());
        }
        _ => panic!("второе событие — Validated, но получено {:?}", events[1]),
    }
    assert!(
        matches!(&events[2], EditorEvent::Dialog(DialogEvent::Idle)),
        "третье событие — Idle"
    );
}

#[test]
fn test_pipeline_format_with_validation_error() {
    let (tx, rx) = spawn_data_layer();
    // Ось X без значения — ошибка валидации
    tx.send(EditorCommand::Pipeline(PipelineCommand::Format {
        content: "G0 X".to_string(),
        renumber_step: 0,
        skip_empty_lines: true,
    }))
    .unwrap();

    // Ждём: NotifyUser(Formatting) + NotifyUser(Error) + Idle
    let events: Vec<EditorEvent> = rx.iter().take(3).collect();
    assert_eq!(events.len(), 3);

    match &events[1] {
        EditorEvent::Dialog(DialogEvent::NotifyUser { message, level }) => {
            assert_eq!(*level, NotifyLevel::Error);
            assert!(message.contains("Ошибка"));
        }
        _ => panic!(
            "второе событие — NotifyUser(Error), но получено {:?}",
            events[1]
        ),
    }
}

// -----------------------------------------------------------------------
// File: сохранение файла
// -----------------------------------------------------------------------

#[test]
fn test_file_save_with_path() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test.nc");

    let (tx, rx) = spawn_data_layer();
    tx.send(EditorCommand::File(FileCommand::SaveFile {
        path: Some(file_path.to_string_lossy().to_string()),
        content: "G0 X10 Y20".to_string(),
    }))
    .unwrap();

    // Ждём: File(Saved) + NotifyUser + Idle
    let events: Vec<EditorEvent> = rx.iter().take(3).collect();
    assert_eq!(events.len(), 3);

    match &events[0] {
        EditorEvent::File(FileEvent::Saved {
            file_path: saved_path,
        }) => {
            assert_eq!(*saved_path, file_path.to_string_lossy());
        }
        _ => panic!("первое событие — Saved, но получено {:?}", events[0]),
    }
    // Файл реально создан на диске
    assert!(file_path.exists());
    let content = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "G0 X10 Y20");
}

#[test]
fn test_file_save_request_picker() {
    // Без path — data layer должен запросить file picker
    let (tx, rx) = spawn_data_layer();
    tx.send(EditorCommand::File(FileCommand::SaveFile {
        path: None,
        content: "G0 X10".to_string(),
    }))
    .unwrap();

    // Ждём: RequestFilePicker
    let event = rx.recv().unwrap();
    match event {
        EditorEvent::Dialog(DialogEvent::RequestFilePicker { mode }) => {
            assert_eq!(mode, FilePickerMode::Save);
        }
        _ => panic!("ожидался RequestFilePicker, но получено {:?}", event),
    }
}

#[test]
fn test_file_save_via_picker_result() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("via_picker.nc");

    let (tx, rx) = spawn_data_layer();
    // 1. Сначала отправляем SaveFile без пути
    tx.send(EditorCommand::File(FileCommand::SaveFile {
        path: None,
        content: "G0 X10".to_string(),
    }))
    .unwrap();

    // 2. Получаем RequestFilePicker
    let _picker = rx.recv().unwrap();

    // 3. Отправляем результат выбора файла
    tx.send(EditorCommand::Dialog(DialogCommand::FilePickerResult {
        path: Some(file_path.to_string_lossy().to_string()),
        mode: FilePickerMode::Save,
    }))
    .unwrap();

    // 4. Ждём: File(Saved) + NotifyUser + Idle
    let events: Vec<EditorEvent> = rx.iter().take(3).collect();
    assert_eq!(events.len(), 3);
    assert!(
        matches!(&events[0], EditorEvent::File(FileEvent::Saved { .. })),
        "первое событие — Saved"
    );
    assert!(file_path.exists());
}

// -----------------------------------------------------------------------
// File: открытие файла
// -----------------------------------------------------------------------

#[test]
fn test_file_open_request_picker() {
    let (tx, rx) = spawn_data_layer();
    tx.send(EditorCommand::File(FileCommand::OpenFile)).unwrap();

    let event = rx.recv().unwrap();
    match event {
        EditorEvent::Dialog(DialogEvent::RequestFilePicker { mode }) => {
            assert_eq!(mode, FilePickerMode::Open);
        }
        _ => panic!("ожидался RequestFilePicker(Open), но получено {:?}", event),
    }
}

#[test]
fn test_file_open_via_picker_result() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("source.nc");
    std::fs::write(&file_path, "G0 X10 Y20").unwrap();

    let (tx, rx) = spawn_data_layer();
    // 1. Запрашиваем открытие
    tx.send(EditorCommand::File(FileCommand::OpenFile)).unwrap();
    let _picker = rx.recv().unwrap();

    // 2. Возвращаем путь из picker
    tx.send(EditorCommand::Dialog(DialogCommand::FilePickerResult {
        path: Some(file_path.to_string_lossy().to_string()),
        mode: FilePickerMode::Open,
    }))
    .unwrap();

    // 3. Ждём: File(Loaded) + Idle
    let events: Vec<EditorEvent> = rx.iter().take(2).collect();
    assert_eq!(events.len(), 2);
    match &events[0] {
        EditorEvent::File(FileEvent::Loaded {
            content,
            file_path: loaded_path,
        }) => {
            assert!(content.contains("G0"));
            assert_eq!(*loaded_path, file_path.to_string_lossy());
        }
        _ => panic!("первое событие — Loaded, но получено {:?}", events[0]),
    }
}

#[test]
fn test_file_open_none_selected() {
    // Пользователь отменил выбор файла (path = None)
    let (tx, rx) = spawn_data_layer();
    tx.send(EditorCommand::File(FileCommand::OpenFile)).unwrap();
    let _picker = rx.recv().unwrap();

    tx.send(EditorCommand::Dialog(DialogCommand::FilePickerResult {
        path: None,
        mode: FilePickerMode::Open,
    }))
    .unwrap();

    // Должен быть только Idle — файл не выбран
    let event = rx.recv().unwrap();
    assert!(
        matches!(event, EditorEvent::Dialog(DialogEvent::Idle)),
        "ожидался Idle при отмене выбора файла"
    );
}

// -----------------------------------------------------------------------
// Dialog: результат диалога
// -----------------------------------------------------------------------

#[test]
fn test_dialog_result() {
    // Диалог ничего не шлёт в ответ (обрабатывается в UI)
    // Главное — не паниковать и не зависнуть
    let (tx, _rx) = spawn_data_layer();
    tx.send(EditorCommand::Dialog(DialogCommand::DialogResult {
        confirmed: false,
    }))
    .unwrap();
    drop(tx);
    // Ждём завершения потока
    std::thread::sleep(std::time::Duration::from_millis(50));
}

// -----------------------------------------------------------------------
// Coalesce: TextChanged не обрабатывается, только последний
// -----------------------------------------------------------------------

#[test]
fn test_textchanged_coalesce() {
    // TextChanged ничего не делает (логика — позже)
    // Проверяем, что data thread не падает
    let (tx, _rx) = spawn_data_layer();
    tx.send(EditorCommand::Pipeline(PipelineCommand::TextChanged(
        "G0 X10".to_string(),
    )))
    .unwrap();
    drop(tx);
    std::thread::sleep(std::time::Duration::from_millis(50));
}
