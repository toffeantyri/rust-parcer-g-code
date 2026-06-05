//! Прикладной слой: use cases приложения
//!
//! Содержит парсер (токены -> AST) и форматтер (AST -> строка).

mod formatter;
mod parser;

pub use formatter::{FormatConfig, Formatter};
pub use parser::Parser;
