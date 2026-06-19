//! Android-бин. Точка входа ANativeActivity_onCreate находится в lib.rs.
//! Этот файл нужен только для Cargo-бина.

#[cfg(not(target_os = "android"))]
fn main() {}
