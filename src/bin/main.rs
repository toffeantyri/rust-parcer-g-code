//use code_parser::{lexer, parser, formatter};

fn main() {
    println!("🚀 Запуск парсера и форматтера...");

    let input = "пример кода для разбора";

    // Пока используем заглушки — нужно убедиться, что модули экспортируют эти функции
    let tokens = lexer::lex(input);
    let ast = parser::parse(&tokens);
    let formatted = formatter::format(&ast);

    println!("Отформатированный код: {}", formatted);
}

// Временные заглушки, если в библиотеке нет реализации
// Уберите это, когда реализуете реальные модули
mod lexer {
    pub fn lex(input: &str) -> Vec<String> {
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
