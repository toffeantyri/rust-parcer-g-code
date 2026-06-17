use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statement_display() {
        let motion = Statement::Motion(MotionStatement {
            code: 0,
            rapid: true,
        });
        let axis = Statement::Axis(AxisStatement {
            axis: "X".to_string(),
            value: Some(10.5),
            decimal_places: Some(1),
        });
        let comment = Statement::Comment(CommentStatement {
            text: "Test move".to_string(),
        });

        assert_eq!(motion.to_string(), "G0");
        assert_eq!(axis.to_string(), "X10.5");
        assert_eq!(comment.to_string(), ";Test move");
    }

    #[test]
    fn test_statement_display_all_variants() {
        let motion = Statement::Motion(MotionStatement {
            code: 1,
            rapid: false,
        });
        assert_eq!(motion.to_string(), "G1");

        let ncode = Statement::NCode(105);
        assert_eq!(ncode.to_string(), "N0105");

        let word = Statement::Word("MODECHECK(2)".to_string());
        assert_eq!(word.to_string(), "MODECHECK(2)");

        let misc = Statement::Misc(MiscStatement { code: 3 });
        assert_eq!(misc.to_string(), "M3");

        let axis_none = Statement::Axis(AxisStatement {
            axis: "X".to_string(),
            value: None,
            decimal_places: None,
        });
        assert_eq!(axis_none.to_string(), "X");

        let raw = Statement::Raw("CFTCP".to_string());
        assert_eq!(raw.to_string(), "CFTCP");

        let newline = Statement::NewLine;
        assert_eq!(newline.to_string(), "\n");
    }

    #[test]
    fn test_while_block_display() {
        let w = Statement::WhileBlock(WhileStatement {
            condition: "R101<R103".to_string(),
            body: vec![],
        });
        assert_eq!(w.to_string(), "WHILE R101<R103");
    }

    #[test]
    fn test_if_block_display() {
        let i = Statement::IfBlock(IfStatement {
            condition: "R101==0".to_string(),
            then_body: vec![],
            else_body: None,
        });
        assert_eq!(i.to_string(), "IF R101==0");
    }
}
