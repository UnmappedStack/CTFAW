mod lexer;
mod parser;
mod ast;
mod error;

fn main() {
    let input: &str = "num = 69;"; // just as a test
    println!("Full input: {}", input);

    let tokens = lexer::lex(input);
    parser::parse_statement(tokens);
}
