mod lexer;
mod parser;
mod error;

fn main() {
    let input: &str = "let val = 211 + 12 / num;"; // just as a test
    println!("Full input: {}", input);
    lexer::lex(input);
}
