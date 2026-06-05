//! Доменные сущности для представления программы G-кода

use std::fmt;

/// Основной оператор программы G-кода
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Перенос строки
    NewLine,
    /// Команда движения (G-коды)
    Motion(MotionStatement),
    /// Номер кадра (N0100, N0105...)
    NCode(i32),
    /// Словесная команда (MODECHECK(2), TRANS, MATLCH(...), CFTCP и т.д.)
    Word(String),
    /// Вспомогательная функция (M-коды)
    Misc(MiscStatement),
    /// Установка координаты оси
    Axis(AxisStatement),
    /// Комментарий
    Comment(CommentStatement),
    /// Неизвестная / сырая конструкция
    Raw(String),
}

/// Команда движения (G0, G1, G2...)
#[derive(Debug, Clone, PartialEq)]
pub struct MotionStatement {
    pub code: i32,
    pub rapid: bool,
}

/// Вспомогательная функция (M3, M5...)
#[derive(Debug, Clone, PartialEq)]
pub struct MiscStatement {
    pub code: i32,
}

/// Координата оси (X10.5, Y20.0...)
#[derive(Debug, Clone, PartialEq)]
pub struct AxisStatement {
    pub axis: String,
    pub value: f64,
}

/// Комментарий
#[derive(Debug, Clone, PartialEq)]
pub struct CommentStatement {
    pub text: String,
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Statement::Motion(m) => write!(f, "G{}", m.code),
            Statement::NCode(code) => write!(f, "N{:04}", code),
            Statement::Word(word) => write!(f, "{}", word),
            Statement::Misc(m) => write!(f, "M{}", m.code),
            Statement::Axis(a) => write!(f, "{}{}", a.axis, a.value),
            Statement::Comment(c) => write!(f, ";{}", c.text),
            Statement::Raw(r) => write!(f, "{}", r),
            Statement::NewLine => write!(f, "\n"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statement_display() {
        let motion = Statement::Motion(MotionStatement {
            code: 0,
            rapid: true,
        });
        let axis = Statement::Axis(AxisStatement {
            axis: "X".to_string(),
            value: 10.5,
        });
        let comment = Statement::Comment(CommentStatement {
            text: "Test move".to_string(),
        });

        assert_eq!(motion.to_string(), "G0");
        assert_eq!(axis.to_string(), "X10.5");
        assert_eq!(comment.to_string(), ";Test move");
    }
}
