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
                    write!(f, "{}{}", a.axis, v)
                } else {
                    write!(f, "{}", a.axis)
                }
            }
            Statement::Comment(c) => write!(f, ";{}", c.text),
            Statement::Raw(r) => write!(f, "{}", r),
            Statement::NewLine => write!(f, "\n"),
            Statement::WhileBlock(w) => write!(f, "WHILE {}", w.condition),
            Statement::IfBlock(i) => write!(f, "IF {}", i.condition),
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
            value: Some(10.5),
        });
        let comment = Statement::Comment(CommentStatement {
            text: "Test move".to_string(),
        });

        assert_eq!(motion.to_string(), "G0");
        assert_eq!(axis.to_string(), "X10.5");
        assert_eq!(comment.to_string(), ";Test move");
    }

    #[test]
    fn test_statement_display_all_variants() {
        let motion = Statement::Motion(MotionStatement {
            code: 1,
            rapid: false,
        });
        assert_eq!(motion.to_string(), "G1");

        let ncode = Statement::NCode(105);
        assert_eq!(ncode.to_string(), "N0105");

        let word = Statement::Word("MODECHECK(2)".to_string());
        assert_eq!(word.to_string(), "MODECHECK(2)");

        let misc = Statement::Misc(MiscStatement { code: 3 });
        assert_eq!(misc.to_string(), "M3");

        let axis_none = Statement::Axis(AxisStatement {
            axis: "X".to_string(),
            value: None,
        });
        assert_eq!(axis_none.to_string(), "X");

        let raw = Statement::Raw("CFTCP".to_string());
        assert_eq!(raw.to_string(), "CFTCP");

        let newline = Statement::NewLine;
        assert_eq!(newline.to_string(), "\n");
    }

    #[test]
    fn test_while_block_display() {
        let w = Statement::WhileBlock(WhileStatement {
            condition: "R101<R103".to_string(),
            body: vec![],
        });
        assert_eq!(w.to_string(), "WHILE R101<R103");
    }

    #[test]
    fn test_if_block_display() {
        let i = Statement::IfBlock(IfStatement {
            condition: "R101==0".to_string(),
            then_body: vec![],
            else_body: None,
        });
        assert_eq!(i.to_string(), "IF R101==0");
    }
}
