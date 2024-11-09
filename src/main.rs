mod lexer;
mod parser;
mod ast;
mod error;

fn main() {
    let input: &str = "(24 + 3) * 4"; // just as a test
    println!("Full input: {}", input);

    let tokens = lexer::lex(input);
    let ast = ast::parse_expression(tokens);

    println!("AST generated:");
    ast::print_ast(&ast);
}
