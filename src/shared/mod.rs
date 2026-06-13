//! Общие утилиты, конфиги и типы ошибок

mod error;
pub mod i18n;

pub use error::{ParseError, Severity, ValidationMessage};
