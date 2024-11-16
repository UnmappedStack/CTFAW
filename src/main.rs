mod lexer;
mod parser;
mod statements;
mod ast;
mod backend;
mod error;

fn main() {
    let input: &str = "23 * 4 - 3 / 4"; // just as a test
    println!("Full input: {}", input);

    let tokens = lexer::lex(input);
    let ast = ast::parse_expression(tokens);

    backend::compile_expression(ast);
}
