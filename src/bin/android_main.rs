//! Android-бин. Точка входа ANativeActivity_onCreate находится в lib.rs.
//! Этот файл нужен только для Cargo-бина.
//! На Android native-activity сам вызывает android_main (из lib.rs).

fn main() {
    // На Android точка входа — android_main из lib.rs.
    // Этот main() никогда не вызывается на Android,
    // но нужен для Cargo чтобы считать бинарником.
}
