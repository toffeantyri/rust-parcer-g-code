//! Intent — намерения пользователя

/// Намерение пользователя — что он хочет сделать
#[derive(Debug, Clone)]
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
    /// Открыть диалог поиска
    ToggleSearch,
    /// Открыть диалог замены
    ToggleReplace,
    /// Выполнить поиск (из диалога поиска)
    DoSearch,
    /// Найти следующее вхождение
    FindNext,
    /// Найти и заменить текущее вхождение
    ReplaceOne,
    /// Заменить все вхождения
    ReplaceAll,
    /// Закрыть диалог поиска/замены
    CloseSearchDialog,
    /// Установить поисковый запрос (из поля ввода)
    SetSearchQuery(String),
    /// Установить строку поиска в диалоге замены
    SetReplaceFind(String),
    /// Установить строку замены в диалоге замены
    SetReplaceWith(String),
    /// Выполнить поиск в диалоге замены
    DoReplaceSearch,
    /// Найти следующее вхождение в диалоге замены
    ReplaceFindNext,
}
