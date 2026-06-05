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
    /// Ось с числовым значением (X10.5, Y-20)
    Axis(String, f64),
    /// Ось с алгебраическим выражением (Z=71.304, X=160+10)
    AxisExpr(String, String),
    Number(f64),
    /// Комментарий после `;`
    Comment(String),
    Eof,
    NewLine,
    Unknown(char),
}
