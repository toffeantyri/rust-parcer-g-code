//! Intent — намерения пользователя

/// Намерение пользователя — что он хочет сделать
#[derive(Debug)]
pub enum Intent {
    OpenFile,
    SaveFile,
    SaveAs,
    CloseFile,
    Exit,
    Format,
    Validate,
}
