//! Прикладной слой: use cases приложения
//!
//! Содержит парсер (токены -> AST), валидатор (AST -> ошибки) и форматтер (AST -> строка).

mod formatter;
mod parser;
mod validator;

pub use formatter::{FormatConfig, Formatter};
pub use parser::Parser;
pub use validator::validate;
