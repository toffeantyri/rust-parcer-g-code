//! Прикладной слой: форматтер — преобразует AST в отформатированный текст G-кода

use crate::domain::{IfStatement, Statement, WhileStatement};

pub struct FormatConfig {
    pub indent_size: usize,
    pub use_spaces: bool,
    pub uppercase_codes: bool,
    pub decimal_places: usize,
    pub renumber_step: u32,
    pub skip_empty_lines: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        FormatConfig {
            indent_size: 2,
            use_spaces: true,
            uppercase_codes: true,
            decimal_places: 5,
            renumber_step: 0,
            skip_empty_lines: true,
        }
    }
}

pub struct Formatter {
    config: FormatConfig,
}

impl Formatter {
    pub fn new(config: FormatConfig) -> Self {
        Formatter { config }
    }

    pub fn format_program(&self, program: &[Statement]) -> String {
        self.format_block(program, 0)
    }

    fn format_block(&self, program: &[Statement], indent_level: usize) -> String {
        let mut result = String::new();
        let mut line_parts: Vec<String> = Vec::new();
        let mut current_n: u32 = 0;
        let mut line_has_ncode = false;

        for stmt in program {
            match stmt {
                Statement::WhileBlock(w) => {
                    // Собираем префикс из текущей строки (обычно N-код)
                    let prefix = if !line_parts.is_empty() {
                        let p = line_parts.join(" ");
                        line_parts.clear();
                        line_has_ncode = false;
                        Some(p)
                    } else {
                        None
                    };
                    result.push_str(&self.format_while(w, indent_level, prefix));
                }
                Statement::IfBlock(i) => {
                    let prefix = if !line_parts.is_empty() {
                        let p = line_parts.join(" ");
                        line_parts.clear();
                        line_has_ncode = false;
                        Some(p)
                    } else {
                        None
                    };
                    result.push_str(&self.format_if(i, indent_level, prefix));
                }
                Statement::NewLine => {
                    if !line_parts.is_empty() {
                        let has_only_ncode = line_has_ncode && line_parts.len() == 1;
                        if self.config.renumber_step > 0
                            && has_only_ncode
                            && self.config.skip_empty_lines
                        {
                            line_parts.clear();
                            result.push('\n');
                        } else {
                            result.push_str(&self.format_line(&line_parts));
                            line_parts.clear();
                        }
                        line_has_ncode = false;
                    } else {
                        if self.config.renumber_step > 0 && !self.config.skip_empty_lines {
                            current_n += self.config.renumber_step;
                            result.push_str(&format!("N{}\n", current_n));
                        } else {
                            result.push('\n');
                        }
                    }
                }
                Statement::NCode(_code) => {
                    if self.config.renumber_step > 0 {
                        current_n += self.config.renumber_step;
                        if !line_parts.is_empty() {
                            result.push_str(&self.format_line(&line_parts));
                            line_parts.clear();
                        }
                        line_parts.push(format!("N{}", current_n));
                        line_has_ncode = true;
                    } else {
                        if !line_parts.is_empty() {
                            result.push_str(&self.format_line(&line_parts));
                            line_parts.clear();
                        }
                        line_parts.push(self.format_statement(stmt));
                        line_has_ncode = true;
                    }
                }
                Statement::Raw(raw) => {
                    if !line_parts.is_empty() {
                        result.push_str(&self.format_line(&line_parts));
                        line_parts.clear();
                        line_has_ncode = false;
                    }
                    result.push_str(raw);
                    result.push('\n');
                }
                _ => {
                    if self.config.renumber_step > 0 && line_parts.is_empty() && !line_has_ncode {
                        current_n += self.config.renumber_step;
                        line_parts.push(format!("N{}", current_n));
                        line_has_ncode = true;
                    }
                    line_parts.push(self.format_statement(stmt));
                }
            }
        }

        if !line_parts.is_empty() {
            result.push_str(&self.format_line(&line_parts));
        }

        result
    }

    fn format_while(
        &self,
        w: &WhileStatement,
        indent_level: usize,
        line_prefix: Option<String>,
    ) -> String {
        let indent = self.make_indent(indent_level);
        let body_indent = self.make_indent(indent_level + 1);
        let mut out = String::new();

        // WHILE на одной строке с N-кодом (если есть)
        let while_line = if let Some(ref prefix) = line_prefix {
            format!("{}{prefix} WHILE {}", indent, w.condition)
        } else {
            format!("{}WHILE {}", indent, w.condition)
        };
        out.push_str(&format!("{}\n", while_line));

        // Тело
        let body = self.format_block(&w.body, indent_level + 1);
        for line in body.lines() {
            if line.trim().is_empty() {
                out.push('\n');
            } else {
                out.push_str(&format!("{}{}\n", body_indent, line));
            }
        }

        // ENDWHILE — если есть N-код от пустой строки внутри тела, он уже в body
        out.push_str(&format!("{}ENDWHILE\n", indent));

        out
    }

    fn format_if(
        &self,
        i: &IfStatement,
        indent_level: usize,
        line_prefix: Option<String>,
    ) -> String {
        let indent = self.make_indent(indent_level);
        let body_indent = self.make_indent(indent_level + 1);
        let mut out = String::new();

        // IF на одной строке с N-кодом
        let if_line = if let Some(ref prefix) = line_prefix {
            format!("{}{prefix} IF {}", indent, i.condition)
        } else {
            format!("{}IF {}", indent, i.condition)
        };
        out.push_str(&format!("{}\n", if_line));

        // THEN
        let then_body = self.format_block(&i.then_body, indent_level + 1);
        for line in then_body.lines() {
            if line.trim().is_empty() {
                out.push('\n');
            } else {
                out.push_str(&format!("{}{}\n", body_indent, line));
            }
        }

        // ELSE
        if let Some(ref else_body) = i.else_body {
            out.push_str(&format!("{}ELSE\n", indent));
            let else_fmt = self.format_block(else_body, indent_level + 1);
            for line in else_fmt.lines() {
                if line.trim().is_empty() {
                    out.push('\n');
                } else {
                    out.push_str(&format!("{}{}\n", body_indent, line));
                }
            }
        }

        // ENDIF
        out.push_str(&format!("{}ENDIF\n", indent));

        out
    }

    fn make_indent(&self, level: usize) -> String {
        let size = self.config.indent_size * level;
        if self.config.use_spaces {
            " ".repeat(size)
        } else {
            "\t".repeat(level)
        }
    }

    fn format_statement(&self, stmt: &Statement) -> String {
        match stmt {
            Statement::Motion(m) => {
                let prefix = if self.config.uppercase_codes { "G" } else { "g" };
                format!("{}{}", prefix, m.code)
            }
            Statement::NCode(code) => format!("N{:04}", code),
            Statement::Word(word) => word.clone(),
            Statement::Misc(m) => {
                let prefix = if self.config.uppercase_codes { "M" } else { "m" };
                format!("{}{}", prefix, m.code)
            }
            Statement::Axis(a) => {
                if let Some(v) = a.value {
                    format!("{}{:.prec$}", a.axis, v, prec = self.config.decimal_places)
                } else {
                    a.axis.clone()
                }
            }
            Statement::Comment(c) => format!(";{}", c.text),
            Statement::Raw(raw) => raw.clone(),
            Statement::WhileBlock(w) => format!("WHILE {}", w.condition),
            Statement::IfBlock(i) => format!("IF {}", i.condition),
            Statement::NewLine => String::new(),
        }
    }

    fn format_line(&self, parts: &[String]) -> String {
        format!("{}\n", parts.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::*;

    #[test]
    fn test_format_simple_program() {
        let program = vec![
            Statement::Motion(MotionStatement { code: 0, rapid: true }),
            Statement::Axis(AxisStatement { axis: "X".to_string(), value: Some(10.0) }),
            Statement::Axis(AxisStatement { axis: "Y".to_string(), value: Some(20.0) }),
            Statement::NewLine,
            Statement::Motion(MotionStatement { code: 1, rapid: false }),
            Statement::Axis(AxisStatement { axis: "Z".to_string(), value: Some(5.5) }),
            Statement::Axis(AxisStatement { axis: "F".to_string(), value: Some(100.0) }),
        ];
        let formatter = Formatter::new(FormatConfig::default());
        let result = formatter.format_program(&program);
        assert_eq!(result, "G0 X10.00000 Y20.00000\nG1 Z5.50000 F100.00000\n");
    }

    #[test]
    fn test_format_ncode() {
        let program = vec![
            Statement::NCode(100),
            Statement::Motion(MotionStatement { code: 0, rapid: true }),
            Statement::NewLine,
            Statement::NCode(105),
            Statement::Motion(MotionStatement { code: 1, rapid: false }),
        ];
        let formatter = Formatter::new(FormatConfig::default());
        let result = formatter.format_program(&program);
        assert_eq!(result, "N0100 G0\nN0105 G1\n");
    }

    #[test]
    fn test_format_with_raw() {
        let program = vec![
            Statement::Motion(MotionStatement { code: 64, rapid: false }),
            Statement::Raw("CFTCP".to_string()),
            Statement::NewLine,
            Statement::Raw("MODECHECK".to_string()),
            Statement::Comment(CommentStatement { text: "2".to_string() }),
            Statement::NewLine,
            Statement::Raw("MAMILL".to_string()),
            Statement::NewLine,
            Statement::Motion(MotionStatement { code: 0, rapid: true }),
        ];
        let formatter = Formatter::new(FormatConfig::default());
        let result = formatter.format_program(&program);
        assert!(result.contains("G64"));
        assert!(result.contains("CFTCP"));
    }

    #[test]
    fn test_format_empty_lines() {
        let program = vec![
            Statement::NewLine,
            Statement::NewLine,
            Statement::Motion(MotionStatement { code: 0, rapid: true }),
        ];
        let formatter = Formatter::new(FormatConfig::default());
        let result = formatter.format_program(&program);
        assert_eq!(result, "\n\nG0\n");
    }

    #[test]
    fn test_renumber_step() {
        let program = vec![
            Statement::NCode(100),
            Statement::Motion(MotionStatement { code: 0, rapid: true }),
            Statement::NewLine,
            Statement::NewLine,
            Statement::NCode(999),
            Statement::Motion(MotionStatement { code: 1, rapid: false }),
        ];
        let config = FormatConfig { renumber_step: 10, skip_empty_lines: true, ..Default::default() };
        let formatter = Formatter::new(config);
        let result = formatter.format_program(&program);
        assert_eq!(result, "N10 G0\n\nN20 G1\n");
    }

    #[test]
    fn test_renumber_skip_empty_false() {
        let program = vec![
            Statement::NCode(1),
            Statement::Motion(MotionStatement { code: 0, rapid: true }),
            Statement::NewLine,
            Statement::NewLine,
            Statement::NCode(2),
            Statement::Motion(MotionStatement { code: 1, rapid: false }),
        ];
        let config = FormatConfig { renumber_step: 1, skip_empty_lines: false, ..Default::default() };
        let formatter = Formatter::new(config);
        let result = formatter.format_program(&program);
        assert_eq!(result, "N1 G0\nN2\nN3 G1\n");
    }

    #[test]
    fn test_renumber_adds_ncode_to_lines_without() {
        let program = vec![
            Statement::Motion(MotionStatement { code: 0, rapid: true }),
            Statement::Axis(AxisStatement { axis: "X".to_string(), value: Some(10.0) }),
            Statement::NewLine,
            Statement::Motion(MotionStatement { code: 1, rapid: false }),
        ];
        let config = FormatConfig { renumber_step: 1, skip_empty_lines: true, ..Default::default() };
        let formatter = Formatter::new(config);
        let result = formatter.format_program(&program);
        assert!(result.starts_with("N1 G0"));
        assert!(result.contains("\nN2 G1"));
    }

    #[test]
    fn test_renumber_removes_empty_ncode_lines() {
        let program = vec![
            Statement::NCode(100),
            Statement::NewLine,
            Statement::NCode(200),
            Statement::Motion(MotionStatement { code: 0, rapid: true }),
        ];
        let config = FormatConfig { renumber_step: 10, skip_empty_lines: true, ..Default::default() };
        let formatter = Formatter::new(config);
        let result = formatter.format_program(&program);
        assert_eq!(result, "\nN20 G0\n");
    }

    #[test]
    fn test_format_while_block() {
        let program = vec![Statement::WhileBlock(WhileStatement {
            condition: "R101<R103".into(),
            body: vec![
                Statement::Motion(MotionStatement { code: 1, rapid: false }),
                Statement::Axis(AxisStatement { axis: "X".into(), value: Some(10.0) }),
                Statement::NewLine,
            ],
        })];
        let formatter = Formatter::new(FormatConfig::default());
        let result = formatter.format_program(&program);
        assert!(result.contains("WHILE R101<R103\n"));
        assert!(result.contains("  G1 X10.00000\n"));
        assert!(result.contains("ENDWHILE\n"));
    }

    #[test]
    fn test_format_while_with_ncode() {
        // WHILE должен быть на одной строке с N-кодом
        let program = vec![
            Statement::NCode(230),
            Statement::WhileBlock(WhileStatement {
                condition: "R101<R103".into(),
                body: vec![
                    Statement::Motion(MotionStatement { code: 1, rapid: false }),
                    Statement::NewLine,
                ],
            }),
        ];
        let formatter = Formatter::new(FormatConfig::default());
        let result = formatter.format_program(&program);
        assert!(result.contains("N0230 WHILE R101<R103\n"), "N-код должен быть на строке WHILE:\n{}", result);
        assert!(result.contains("  G1\n"));
        assert!(result.contains("ENDWHILE\n"));
    }

    #[test]
    fn test_format_if_else_block() {
        let program = vec![Statement::IfBlock(IfStatement {
            condition: "R101==0".into(),
            then_body: vec![
                Statement::Motion(MotionStatement { code: 0, rapid: true }),
                Statement::Axis(AxisStatement { axis: "X".into(), value: Some(10.0) }),
                Statement::NewLine,
            ],
            else_body: Some(vec![
                Statement::Motion(MotionStatement { code: 1, rapid: false }),
                Statement::Axis(AxisStatement { axis: "Y".into(), value: Some(20.0) }),
                Statement::NewLine,
            ]),
        })];
        let formatter = Formatter::new(FormatConfig::default());
        let result = formatter.format_program(&program);
        assert!(result.contains("IF R101==0\n"));
        assert!(result.contains("  G0 X10.00000\n"));
        assert!(result.contains("ELSE\n"));
        assert!(result.contains("  G1 Y20.00000\n"));
        assert!(result.contains("ENDIF\n"));
    }

    #[test]
    fn test_format_nested_while() {
        let program = vec![Statement::WhileBlock(WhileStatement {
            condition: "R101<R103".into(),
            body: vec![
                Statement::WhileBlock(WhileStatement {
                    condition: "R102<R103".into(),
                    body: vec![
                        Statement::Motion(MotionStatement { code: 1, rapid: false }),
                        Statement::Axis(AxisStatement { axis: "X".into(), value: Some(10.0) }),
                        Statement::NewLine,
                    ],
                }),
                Statement::NewLine,
            ],
        })];
        let formatter = Formatter::new(FormatConfig::default());
        let result = formatter.format_program(&program);
        assert!(result.contains("WHILE R101<R103\n"));
        assert!(result.contains("  WHILE R102<R103\n"));
        assert!(result.contains("    G1 X10.00000\n"));
        assert!(result.contains("  ENDWHILE\n"));
        assert!(result.contains("ENDWHILE\n"));
    }
}
