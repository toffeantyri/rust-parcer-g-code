use super::*;

#[test]
fn test_format_settings_default() {
    let s = FormatSettings::default();
    assert_eq!(s.renumber_step, 1);
    assert!(s.skip_empty_lines);
    assert_eq!(s.language, "ru");
}

#[test]
fn test_model_default() {
    let m = Model::default();
    assert!(m.content().is_empty());
    assert!(m.file_path().is_empty());
    assert!(!m.is_busy());
    assert!(!m.settings_open());
    assert!(!m.modified());
    assert!(!m.show_exit_dialog());
    assert_eq!(m.pending_action(), None);
    assert_eq!(m.save_and_exec(), None);
    assert_eq!(m.format_settings().renumber_step, 1);
}

#[test]
fn test_format_settings_serde_roundtrip() {
    let original = FormatSettings {
        renumber_step: 10,
        skip_empty_lines: false,
        language: "en".to_string(),
    };
    let json = serde_json::to_string(&original).unwrap();
    let restored: FormatSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.renumber_step, 10);
    assert!(!restored.skip_empty_lines);
    assert_eq!(restored.language, "en");
}

#[test]
fn test_format_settings_serde_default_json() {
    // Пустой JSON с полями по умолчанию
    let json = r#"{"renumber_step":1,"skip_empty_lines":true,"language":"ru"}"#;
    let restored: FormatSettings = serde_json::from_str(json).unwrap();
    assert_eq!(restored, FormatSettings::default());
}

#[test]
fn test_format_settings_serde_custom_json() {
    let json = r#"{"renumber_step":100,"skip_empty_lines":false,"language":"en"}"#;
    let restored: FormatSettings = serde_json::from_str(json).unwrap();
    assert_eq!(restored.renumber_step, 100);
    assert!(!restored.skip_empty_lines);
    assert_eq!(restored.language, "en");
}
