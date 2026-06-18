//! Доменные типы токенов, используемые лексером

/// Токен — элементарная единица разбора G-кода
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    GCode(i32),
    MCode(i32),
    /// Номер кадра (N0100, N0105...)
    NCode(i32),
    /// Многосимвольное слово (MODECHECK, TRANS, MATLCH, CFTCP и т.д.)
    Word(String),
    /// Ось с числовым значением (X10.5, Y-20) или без значения (X — ошибка).
    /// Параметры: (буква, значение, знаки_после_запятой)
    /// decimal_places: None = целое без точки, Some(n) = n знаков после запятой
    Axis(String, Option<f64>, Option<usize>),
    /// Ось с алгебраическим выражением (Z=71.304, X=160+10)
    AxisExpr(String, String),
    /// Скорость вращения шпинделя (S1000, S1=1000, S2=500)
    Speed(String),
    /// R-параметр (R50, R101=R101+1)
    RParameter(String),
    Number(f64),
    /// Комментарий после `;`
    Comment(String),
    /// Управляющие конструкции потока
    /// WhileBlock { condition: String } — условие цикла
    /// Открывается EndWhile
    WhileBlock(String),
    /// EndWhile — закрывает While (без условия)
    EndWhile,
    /// IfBlock { condition: String } — условие в if
    /// Открывается EndIf
    IfBlock(String),
    /// Else — ветка else внутри if (без условия)
    Else,
    /// EndIf — закрывает if (без условия)
    EndIf,
    /// Repeat — начало цикла с пост-условием,
    /// закрывается Until
    Repeat,
    /// Until { condition: String } — закрывает Repeat
    Until(String),
    /// For — начало цикла со счётчиком
    For(String),
    /// EndFor — закрывает For
    EndFor,
    /// Loop — начало бесконечного цикла
    LoopBlock(String),
    /// EndLoop — закрывает Loop
    EndLoop,
    Eof,
    NewLine,
    Unknown(char),
}
