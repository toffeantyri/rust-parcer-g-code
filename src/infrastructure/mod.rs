//! Инфраструктурный слой: реализация внешних зависимостей
//!
//! Содержит лексер (преобразование текста в токены) и другие адаптеры,
//! реализующие доменные трейты.

pub mod highlight;
pub mod lexer;
#[cfg(not(target_os = "android"))]
pub mod platform;

pub use lexer::DefaultLexer;
