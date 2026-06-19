use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_lang_is_ru() {
        let _lock = test_lock();
        set_lang("ru");
        // Язык по умолчанию — русский
        assert_eq!(current_lang(), "ru");
        assert_eq!(&locale().menu.file, "Файл");
        assert_eq!(
            &locale().status.ready,
            "Готов к работе. Откройте файл G-кода."
        );
    }

    #[test]
    fn test_set_lang_en() {
        let _lock = test_lock();
        set_lang("en");
        assert_eq!(current_lang(), "en");
        assert_eq!(&locale().menu.file, "File");
        assert_eq!(&locale().status.ready, "Ready. Open a G-code file.");
        set_lang("ru");
        assert_eq!(current_lang(), "ru");
    }

    #[test]
    fn test_set_lang_back_and_forth() {
        let _lock = test_lock();
        set_lang("en");
        assert_eq!(current_lang(), "en");
        set_lang("ru");
        assert_eq!(current_lang(), "ru");
        set_lang("en-US");
        assert_eq!(current_lang(), "en");
        set_lang("fr");
        assert_eq!(current_lang(), "ru");
    }

    #[test]
    fn test_locale_switch_affects_all_categories() {
        let _lock = test_lock();
        set_lang("en");
        assert_eq!(&locale().menu.settings, "Settings");
        assert_eq!(&locale().toolbar.open, "Open");
        assert_eq!(&locale().dialog.exit_title, "Save changes?");
        assert_eq!(&locale().settings.title, "Format Settings");
        set_lang("ru");
    }

    #[test]
    fn test_fmt_save_error_ru() {
        let _lock = test_lock();
        set_lang("ru");
        let msg = fmt_save_error("Permission denied");
        assert_eq!(msg, "Ошибка сохранения: Permission denied");
    }

    #[test]
    fn test_fmt_save_error_en() {
        let _lock = test_lock();
        set_lang("en");
        let msg = fmt_save_error("Permission denied");
        assert_eq!(msg, "Save error: Permission denied");
        set_lang("ru");
    }

    #[test]
    fn test_fmt_error_ru() {
        let _lock = test_lock();
        set_lang("ru");
        let msg = fmt_error("Syntax error");
        assert_eq!(msg, "Ошибка: Syntax error");
    }

    #[test]
    fn test_fmt_error_en() {
        let _lock = test_lock();
        set_lang("en");
        let msg = fmt_error("Syntax error");
        assert_eq!(msg, "Error: Syntax error");
        set_lang("ru");
    }

    #[test]
    fn test_fmt_errors_found_ru() {
        let _lock = test_lock();
        set_lang("ru");
        let msg = fmt_errors_found(3, "ось X без значения", 5);
        assert_eq!(
            msg,
            "Найдено 3 ошибок. Первая: ось X без значения [строка 5]"
        );
    }

    #[test]
    fn test_fmt_errors_found_en() {
        let _lock = test_lock();
        set_lang("en");
        let msg = fmt_errors_found(1, "axis X without value", 2);
        assert_eq!(msg, "Found 1 errors. First: axis X without value [line 2]");
        set_lang("ru");
    }

    #[test]
    fn test_fmt_saved_ru() {
        let _lock = test_lock();
        set_lang("ru");
        let msg = fmt_saved("/path/file.nc");
        assert_eq!(msg, "Сохранено: /path/file.nc");
    }

    #[test]
    fn test_fmt_saved_en() {
        let _lock = test_lock();
        set_lang("en");
        let msg = fmt_saved("/path/file.nc");
        assert_eq!(msg, "Saved: /path/file.nc");
        set_lang("ru");
    }
}
