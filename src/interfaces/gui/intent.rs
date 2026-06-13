//! Intent — намерения пользователя

/// Намерение пользователя — что он хочет сделать
#[derive(Debug)]
pub enum Intent {
    OpenFile,
    SaveFile,
    SaveAs,
    CloseFile,
    Exit,
    Format,
    Validate,
    /// Открыть / закрыть окно настроек форматирования
    ToggleSettings,
    /// Установить шаг перенумерации
    SetRenumberStep(u32),
    /// Установить флаг пропуска пустых строк
    SetSkipEmptyLines(bool),
    /// Подтверждение действия в диалоге
    ConfirmSave,
    /// Отказ от сохранения
    DiscardAndContinue,
    /// Отмена действия
    CancelAction,
    /// Открыть / закрыть окно горячих клавиш
    ToggleShortcuts,
    /// Установить язык интерфейса ("ru" или "en")
    SetLanguage(String),
}
