mod lexer;
mod parser;
mod error;

fn main() {
    let input: &str = "(24 + 3) * 4"; // just as a test
    println!("Full input: {}", input);

    let tokens = lexer::lex(input);
    let ast = parser::parse_expression(tokens);

    println!("AST generated:");
    parser::print_ast(&ast);
}
