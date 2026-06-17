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
    /// Блок WHILE ... ENDWHILE с телом и отступами
    WhileBlock(WhileStatement),
    /// Блок IF ... ELSE ... ENDIF
    IfBlock(IfStatement),
    /// Скорость шпинделя (S1000, S1=1000, SS1=500)
    Speed(String),
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
/// Если значение None — ось указана без числа (ошибка: `X` без значения)
#[derive(Debug, Clone, PartialEq)]
pub struct AxisStatement {
    pub axis: String,
    pub value: Option<f64>,
    /// Количество знаков после запятой в исходном тексте.
    /// None = целое число без точки, Some(n) = n знаков после запятой.
    pub decimal_places: Option<usize>,
}

/// Комментарий
#[derive(Debug, Clone, PartialEq)]
pub struct CommentStatement {
    pub text: String,
}

/// Блок WHILE ... ENDWHILE
#[derive(Debug, Clone, PartialEq)]
pub struct WhileStatement {
    /// Условие (например "R101<R103")
    pub condition: String,
    /// Тело цикла — операторы внутри WHILE...ENDWHILE
    pub body: Vec<Statement>,
}

/// Блок IF ... ENDIF с опциональным ELSE
#[derive(Debug, Clone, PartialEq)]
pub struct IfStatement {
    /// Условие (например "R101==0")
    pub condition: String,
    /// Тело IF
    pub then_body: Vec<Statement>,
    /// Тело ELSE (None если ELSE нет)
    pub else_body: Option<Vec<Statement>>,
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Statement::Motion(m) => write!(f, "G{}", m.code),
            Statement::NCode(code) => write!(f, "N{:04}", code),
            Statement::Word(word) => write!(f, "{}", word),
            Statement::Misc(m) => write!(f, "M{}", m.code),
            Statement::Axis(a) => {
                if let Some(v) = a.value {
                    if let Some(prec) = a.decimal_places {
                        write!(f, "{}{:.prec$}", a.axis, v, prec = prec)
                    } else {
                        write!(f, "{}{}", a.axis, v)
                    }
                } else {
                    write!(f, "{}", a.axis)
                }
            }
            Statement::Comment(c) => write!(f, ";{}", c.text),
            Statement::Raw(r) => write!(f, "{}", r),
            Statement::NewLine => write!(f, "\n"),
            Statement::WhileBlock(w) => write!(f, "WHILE {}", w.condition),
            Statement::IfBlock(i) => write!(f, "IF {}", i.condition),
            Statement::Speed(s) => write!(f, "{}", s),
        }
    }
}

#[cfg(test)]
#[path = "ast_tests.rs"]
mod tests;
