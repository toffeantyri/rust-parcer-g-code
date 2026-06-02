use std::fs;
//use code_parser::{lexer, parser, formatter};

fn main() {
    println!("🚀 Запуск парсера и форматтера...");

    let input_content = fs::read_to_string("input_code.txt").expect("Error reading file");
    println!("{}", input_content);

    // Пока используем заглушки — нужно убедиться, что модули экспортируют эти функции
    let tokens = lexer::lex(&input_content);
    let ast = parser::parse(&tokens);
    let formatted = formatter::format(&ast);

    println!("Отформатированный код: {}", formatted);
}

// Временные заглушки, если в библиотеке нет реализации
// Уберите это, когда реализуете реальные модули
mod lexer {

    pub fn lex(input: &str) -> Vec<String> {
        //let lexer = Lexer::new(input.to_string());
        vec![format!("LEXED({})", input)]
    }
}

mod parser {
    pub fn parse(tokens: &[String]) -> String {
        format!("AST({:?})", tokens)
    }
}

mod formatter {
    pub fn format(ast: &str) -> String {
        format!("Formatted: {}", ast)
    }
}
