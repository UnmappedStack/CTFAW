mod lexer;
mod parser;
mod error;

fn main() {
    let input: &str = "24 + 3 * 4 + 7"; // just as a test
    println!("Full input: {}", input);

    let tokens = lexer::lex(input);
    parser::parse_expression(tokens);
}
