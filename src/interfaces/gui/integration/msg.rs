//! Msg — сообщения между View и Data Layer.
//!
//! Аналог Intent, но:
//! - включает события от data layer к UI (Formatted, Validated, Loaded и т.д.)
//! - не знает о mpsc-каналах — только данные
//! - Clone + Send + 'static (требование StateStore)

use crate::interfaces::gui::intent::AxisSwapMode;

/// Сообщение: либо намерение пользователя из View, либо событие от Data Layer.
#[derive(Debug, Clone)]
pub enum Msg {
    // ── Намерения пользователя (из View → Data Layer) ──
    /// Открыть файл
    OpenFile,
    /// Сохранить файл (текущий путь)
    SaveFile,
    /// Сохранить как (новый путь)
    SaveAs,
    /// Закрыть файл
    CloseFile,
    /// Выйти из приложения
    Exit,
    /// Отформатировать
    Format,
    /// Проверить
    Validate,
    /// Открыть/закрыть настройки
    ToggleSettings,
    /// Установить шаг перенумерации
    SetRenumberStep(u32),
    /// Пропускать пустые строки
    SetSkipEmptyLines(bool),
    /// Подтвердить сохранение
    ConfirmSave,
    /// Отказ от сохранения
    DiscardAndContinue,
    /// Отмена
    CancelAction,
    /// Открыть/закрыть шорткаты
    ToggleShortcuts,
    /// Установить язык
    SetLanguage(String),
    /// Открыть/закрыть поиск
    ToggleSearch,
    /// Открыть/закрыть замену
    ToggleReplace,
    /// Выполнить поиск
    DoSearch,
    /// Найти следующее
    FindNext,
    /// Заменить одно
    ReplaceOne,
    /// Заменить все
    ReplaceAll,
    /// Закрыть поиск/замену
    CloseSearchDialog,
    /// Установить поисковый запрос
    SetSearchQuery(String),
    /// Установить строку поиска (замена)
    SetReplaceFind(String),
    /// Установить строку замены
    SetReplaceWith(String),
    /// Выполнить поиск (замена)
    DoReplaceSearch,
    /// Найти следующее (замена)
    ReplaceFindNext,
    /// Открыть/закрыть замену осей
    ToggleAxisSwap,
    /// Установить первую ось для swap
    SetSwapAxis1(String),
    /// Установить вторую ось для swap
    SetSwapAxis2(String),
    /// Установить ось для инвертирования
    SetInvertAxis(String),
    /// Установить режим замены осей
    SetAxisSwapMode(AxisSwapMode),
    /// Применить замену осей
    ApplyAxisSwap,
    /// Открыть/закрыть Drawer
    ToggleDrawer,
    /// Текст изменился (из TextEdit)
    TextChanged(String),

    // ── События от Data Layer (Data Layer → View) ──
    /// Результат форматирования
    Formatted {
        content: String,
        errors: Vec<crate::shared::ValidationMessage>,
    },
    /// Результат валидации
    Validated {
        errors: Vec<crate::shared::ValidationMessage>,
    },
    /// Файл загружен
    FileLoaded { content: String, file_path: String },
    /// Файл сохранён
    FileSaved { file_path: String },
    /// Показать уведомление
    Notify { message: String },
    /// Data layer завершил обработку
    Idle,
}
