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
    Eof,
    NewLine,
    Unknown(char),
}
