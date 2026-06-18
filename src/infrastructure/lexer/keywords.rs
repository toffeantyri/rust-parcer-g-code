//! Словарь ключевых слов G-кода по диалектам.
//! Парсер использует словарь через dependency injection
//! для определения управляющих конструкций и системных команд.

use std::collections::HashSet;

/// Словарь ключевых слов для парсинга G-кода.
/// Определяет, какие слова являются управляющими конструкциями потока
/// (WHILE, IF, и т.д.) и системными командами (MODECHECK, MSG и т.д.).
#[derive(Debug, Clone)]
pub struct KeywordDictionary {
    /// Ключевые слова потока (управляющие конструкции)
    flow_control: HashSet<String>,
    /// Системные команды / многосимвольные инструкции станка
    system_commands: HashSet<String>,
    /// Команды смены инструмента / обработки
    miscellaneous: HashSet<String>,
}

impl KeywordDictionary {
    /// Пустой словарь (ничего не распознаёт как ключевое слово)
    pub fn new() -> Self {
        Self {
            flow_control: HashSet::new(),
            system_commands: HashSet::new(),
            miscellaneous: HashSet::new(),
        }
    }

    /// Словарь Siemens (по умолчанию)
    pub fn siemens() -> Self {
        let mut dict = Self::new();
        dict.flow_control.extend(
            [
                "WHILE", "IF", "ELSE", "ENDIF", "ENDWHILE", "REPEAT", "UNTIL", "FOR", "ENDFOR",
                "LOOP", "ENDLOOP",
            ]
            .iter()
            .map(|&s| s.to_string()),
        );
        dict.system_commands.extend(
            [
                "MODECHECK",
                "STOPRE",
                "SUPA",
                "MSG",
                "SETAL",
                "CFTCP",
                "CFC",
                "TRAFOOF",
                "TRAFOON",
            ]
            .iter()
            .map(|&s| s.to_string()),
        );
        dict.miscellaneous.extend(
            [
                "MAMILL",
                "MATLCH",
                "MATLRET",
                "WGTRANS",
                "WGROTON",
                "WGROTOF",
                "GOTOF",
                "GOTOB",
                "CHECKMAXR",
                "EXTERN",
            ]
            .iter()
            .map(|&s| s.to_string()),
        );
        dict
    }

    /// Проверяет, является ли слово управляющей конструкцией потока
    pub fn is_flow_control(&self, word: &str) -> bool {
        self.flow_control.contains(&word.to_uppercase())
    }

    /// Проверяет, является ли слово системной командой
    pub fn is_system_command(&self, word: &str) -> bool {
        self.system_commands.contains(&word.to_uppercase())
    }

    /// Проверяет, является ли слово вспомогательной командой станка
    /// (которые не надо разбирать на отдельные токены)
    pub fn is_miscellaneous(&self, word: &str) -> bool {
        self.miscellaneous.contains(&word.to_uppercase())
    }

    /// Проверяет, является ли слово известным ключевым словом
    pub fn is_known(&self, word: &str) -> bool {
        self.is_flow_control(word) || self.is_system_command(word) || self.is_miscellaneous(word)
    }
}

impl Default for KeywordDictionary {
    fn default() -> Self {
        Self::siemens()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_control() {
        let dict = KeywordDictionary::siemens();
        assert!(dict.is_flow_control("WHILE"));
        assert!(dict.is_flow_control("while"));
        assert!(dict.is_flow_control("IF"));
        assert!(dict.is_flow_control("else"));
        assert!(dict.is_flow_control("ENDIF"));
        assert!(dict.is_flow_control("ENDWHILE"));
        assert!(dict.is_flow_control("REPEAT"));
        assert!(dict.is_flow_control("UNTIL"));
    }

    #[test]
    fn test_system_commands() {
        let dict = KeywordDictionary::siemens();
        assert!(dict.is_system_command("MODECHECK"));
        assert!(dict.is_system_command("modecheck"));
        assert!(dict.is_system_command("MSG"));
        assert!(dict.is_system_command("CFTCP"));
        assert!(dict.is_system_command("CFC"));
        assert!(!dict.is_system_command("GOTO"));
    }

    #[test]
    fn test_miscellaneous() {
        let dict = KeywordDictionary::siemens();
        assert!(dict.is_miscellaneous("MAMILL"));
        assert!(dict.is_miscellaneous("mamill"));
        assert!(dict.is_miscellaneous("MATLCH"));
        assert!(dict.is_miscellaneous("WGTRANS"));
        assert!(dict.is_miscellaneous("EXTERN"));
        assert!(dict.is_miscellaneous("CHECKMAXR"));
    }

    #[test]
    fn test_unknown_not_known() {
        let dict = KeywordDictionary::siemens();
        assert!(!dict.is_known("G01"));
        assert!(!dict.is_known("X10"));
        assert!(!dict.is_known("G0"));
        assert!(!dict.is_known("M30"));
    }

    #[test]
    fn test_empty_dictionary() {
        let dict = KeywordDictionary::new();
        assert!(!dict.is_flow_control("WHILE"));
        assert!(!dict.is_system_command("MODECHECK"));
        assert!(!dict.is_miscellaneous("MAMILL"));
    }
}
