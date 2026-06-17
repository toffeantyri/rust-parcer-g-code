//! Тесты лексера G-кода

use super::*;

#[test]
fn test_simple_gcode() {
    let tokens = tokenize("G0 X10 Y20 (Rapid move)\nG1 Z5.5 F100");

    assert_eq!(tokens[0], Token::GCode(0));
    assert_eq!(tokens[1], Token::Axis("X".to_string(), Some(10.0), None));
    assert_eq!(tokens[2], Token::Axis("Y".to_string(), Some(20.0), None));
    // Скобки — не комментарий, а скобочное выражение (Word)
    assert_eq!(tokens[3], Token::Word("(Rapid move)".to_string()));
    assert_eq!(tokens[4], Token::NewLine);
    assert_eq!(tokens[5], Token::GCode(1));
    assert_eq!(tokens[6], Token::Axis("Z".to_string(), Some(5.5), Some(1)));
    assert_eq!(tokens[7], Token::Axis("F".to_string(), Some(100.0), None));
}

#[test]
fn test_semicolon_comment() {
    let tokens = tokenize("G0 X10 ; this is a comment\nG1 Y20");

    assert_eq!(tokens[0], Token::GCode(0));
    assert_eq!(tokens[1], Token::Axis("X".to_string(), Some(10.0), None));
    assert_eq!(tokens[2], Token::Comment(" this is a comment".to_string()));
    assert_eq!(tokens[3], Token::NewLine);
    assert_eq!(tokens[4], Token::GCode(1));
    assert_eq!(tokens[5], Token::Axis("Y".to_string(), Some(20.0), None));
}

#[test]
fn test_negative_numbers() {
    let tokens = tokenize("G0 X-10 Y-20.5");

    assert_eq!(tokens[0], Token::GCode(0));
    assert_eq!(tokens[1], Token::Axis("X".to_string(), Some(-10.0), None));
    assert_eq!(
        tokens[2],
        Token::Axis("Y".to_string(), Some(-20.5), Some(1))
    );
}

#[test]
fn test_axis_expr() {
    let tokens = tokenize("Z=71.304 X=160+10 Y=3*5/2");

    assert_eq!(
        tokens[0],
        Token::AxisExpr("Z".to_string(), "71.304".to_string())
    );
    assert_eq!(
        tokens[1],
        Token::AxisExpr("X".to_string(), "160+10".to_string())
    );
    assert_eq!(
        tokens[2],
        Token::AxisExpr("Y".to_string(), "3*5/2".to_string())
    );
}

#[test]
fn test_mcode() {
    let tokens = tokenize("M3 M05 M17");

    assert_eq!(tokens[0], Token::MCode(3));
    assert_eq!(tokens[1], Token::MCode(5));
    assert_eq!(tokens[2], Token::MCode(17));
}

#[test]
fn test_paren_expr() {
    // Скобки — не комментарий, а часть команды (словесной или параметров)
    let tokens = tokenize("G0 (comment (nested) content) X10");

    assert_eq!(tokens[0], Token::GCode(0));
    // Всё в скобках — Word (скобочное выражение)
    assert_eq!(
        tokens[1],
        Token::Word("(comment (nested) content)".to_string())
    );
    assert_eq!(tokens[2], Token::Axis("X".to_string(), Some(10.0), None));
}

#[test]
fn test_multichar_words() {
    let tokens = tokenize("MODECHECK(2) TRANS Z-8 MATLCH(\"DISKD125\",0,1)");

    // Скобочные аргументы — часть слова
    assert_eq!(tokens[0], Token::Word("MODECHECK(2)".to_string()));
    assert_eq!(tokens[1], Token::Word("TRANS".to_string()));
    assert_eq!(tokens[2], Token::Axis("Z".to_string(), Some(-8.0), None));
    assert_eq!(
        tokens[3],
        Token::Word("MATLCH(\"DISKD125\",0,1)".to_string())
    );
}

#[test]
fn test_n_codes() {
    let tokens = tokenize("N0100 G64 CFTCP");

    // N-номер сохраняется как NCode с целым числом
    assert_eq!(tokens[0], Token::NCode(100));
    assert_eq!(tokens[1], Token::GCode(64));
    assert_eq!(tokens[2], Token::Word("CFTCP".to_string()));
}

#[test]
fn test_axis_without_gcode() {
    // Строки начинающиеся с оси (продолжение предыдущего G-кода)
    let tokens = tokenize(" Z71.304\n Y-58.346");

    assert_eq!(
        tokens[0],
        Token::Axis("Z".to_string(), Some(71.304), Some(3))
    );
    assert_eq!(tokens[1], Token::NewLine);
    assert_eq!(
        tokens[2],
        Token::Axis("Y".to_string(), Some(-58.346), Some(3))
    );
}

// -----------------------------------------------------------------------
// Пограничные случаи
// -----------------------------------------------------------------------

#[test]
fn test_word_with_space_before_paren() {
    // Пробел между многосимвольным словом и скобкой
    let tokens = tokenize("MODECHECK (2)");
    assert_eq!(tokens[0], Token::Word("MODECHECK(2)".to_string()));
}

#[test]
fn test_comment_at_end_of_line() {
    // Комментарий после команды на той же строке
    let tokens = tokenize("G0 X10 ;это комментарий\nG1 Y20");

    assert_eq!(tokens[0], Token::GCode(0));
    assert_eq!(tokens[1], Token::Axis("X".to_string(), Some(10.0), None));
    assert_eq!(tokens[2], Token::Comment("это комментарий".to_string()));
    assert_eq!(tokens[3], Token::NewLine);
    assert_eq!(tokens[4], Token::GCode(1));
    assert_eq!(tokens[5], Token::Axis("Y".to_string(), Some(20.0), None));
}

#[test]
fn test_unknown_symbols() {
    // Неизвестные символы не должны вызывать панику
    let tokens = tokenize("@#%");
    assert_eq!(tokens[0], Token::Unknown('@'));
    assert_eq!(tokens[1], Token::Unknown('#'));
    assert_eq!(tokens[2], Token::Unknown('%'));
}

#[test]
fn test_axis_with_negative_value() {
    // Ось с отрицательным числом без пробела, без G-кода
    let tokens = tokenize("X-10");
    assert_eq!(tokens[0], Token::Axis("X".to_string(), Some(-10.0), None));
}

#[test]
fn test_empty_input() {
    // Пустой ввод не должен паниковать
    let tokens = tokenize("");
    assert!(tokens.is_empty());
}

#[test]
fn test_whitespace_only() {
    // Только пробелы без перевода строки
    let tokens = tokenize("   \t  ");
    assert!(tokens.is_empty());
}

#[test]
fn test_whitespace_with_newlines() {
    // Пробелы с переводами строк — NewLine сохраняются
    let tokens = tokenize("  \n  \n");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0], Token::NewLine);
    assert_eq!(tokens[1], Token::NewLine);
}

#[test]
fn test_system_variable() {
    let tokens = tokenize("R50=$TC_MPP6[9998,1]");
    assert_eq!(tokens.len(), 1);
    assert_eq!(
        tokens[0],
        Token::RParameter("R50=$TC_MPP6[9998,1]".to_string())
    );
}

#[test]
fn test_r_parameter_assign() {
    let tokens = tokenize("R101=R101+1");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::RParameter("R101=R101+1".to_string()));
}

#[test]
fn test_r_parameter_assign_with_spaces() {
    let tokens = tokenize("R100 = 5");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::RParameter("R100=5".to_string()));
}

#[test]
fn test_r_parameter_simple() {
    let tokens = tokenize("G0 R50 X100");
    assert_eq!(tokens[0], Token::GCode(0));
    assert_eq!(tokens[1], Token::RParameter("R50".to_string()));
    assert_eq!(tokens[2], Token::Axis("X".to_string(), Some(100.0), None));
}

#[test]
fn test_rcheck_is_word_not_rparameter() {
    // RCHECK — команда, не R-параметр
    let tokens = tokenize("RCHECK");
    assert_eq!(tokens[0], Token::Word("RCHECK".to_string()));
}

#[test]
fn test_rmac_is_word_not_rparameter() {
    // RMAC — команда, не R-параметр
    let tokens = tokenize("RMAC");
    assert_eq!(tokens[0], Token::Word("RMAC".to_string()));
}

#[test]
fn test_r_check() {
    let tokens = tokenize("RCHECK");
    assert_eq!(tokens[0], Token::Word("RCHECK".to_string()));
}

#[test]
fn test_while_condition() {
    let tokens = tokenize("WHILE R101<R103");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Word("WHILE R101<R103".to_string()));
}

#[test]
fn test_while_condition_with_parens() {
    let tokens = tokenize("WHILE (R101<3) AND (R102>0)");
    assert_eq!(tokens.len(), 1);
    assert_eq!(
        tokens[0],
        Token::Word("WHILE (R101<3) AND (R102>0)".to_string())
    );
}

#[test]
fn test_if_condition() {
    let tokens = tokenize("IF R101==0");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Word("IF R101==0".to_string()));
}

#[test]
fn test_endwhile_no_condition() {
    let tokens = tokenize("ENDWHILE");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Word("ENDWHILE".to_string()));
}

#[test]
fn test_endif_no_condition() {
    let tokens = tokenize("ENDIF");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Word("ENDIF".to_string()));
}

#[test]
fn test_else_standalone() {
    let tokens = tokenize("ELSE");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Word("ELSE".to_string()));
}

#[test]
fn test_repeat_condition() {
    let tokens = tokenize("REPEAT R101<R103");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Word("REPEAT R101<R103".to_string()));
}

#[test]
fn test_until_condition() {
    let tokens = tokenize("UNTIL R101>=R103");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Word("UNTIL R101>=R103".to_string()));
}

#[test]
fn test_system_variable_standalone() {
    // Системная переменная без R-параметра
    let tokens = tokenize("$TC_MPP6[9998,1]");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Word("$TC_MPP6[9998,1]".to_string()));
}

#[test]
fn test_while_with_trailing_comment() {
    // WHILE с комментарием — условие не должно включать комментарий
    let tokens = tokenize("WHILE R101<R103 ; loop condition");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0], Token::Word("WHILE R101<R103".to_string()));
    assert_eq!(tokens[1], Token::Comment(" loop condition".to_string()));
}

#[test]
fn test_if_lowercase() {
    // Ключевые слова регистронезависимы
    let tokens = tokenize("if R101==0");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Word("if R101==0".to_string()));
}

#[test]
fn test_until_with_spaces() {
    // UNTIL с пробелами вокруг скобок
    let tokens = tokenize("UNTIL (R101>=R103)");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Word("UNTIL (R101>=R103)".to_string()));
}

#[test]
fn test_if_with_parens() {
    let tokens = tokenize("IF (R101==0) AND (R102>5)");
    assert_eq!(tokens.len(), 1);
    assert_eq!(
        tokens[0],
        Token::Word("IF (R101==0) AND (R102>5)".to_string())
    );
}

#[test]
fn test_repeat_no_condition() {
    let tokens = tokenize("REPEAT");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Word("REPEAT".to_string()));
}

#[test]
fn test_while_no_condition() {
    let tokens = tokenize("WHILE");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Word("WHILE".to_string()));
}

#[test]
fn test_r_alone() {
    let tokens = tokenize("G0 R X100");
    assert_eq!(tokens[0], Token::GCode(0));
    assert_eq!(tokens[1], Token::Word("R".to_string()));
    assert_eq!(tokens[2], Token::Axis("X".to_string(), Some(100.0), None));
}
