mod lexer;
mod error;

fn main() {
    let input: &str = "let 211 + 12;"; // just as a test
    println!("Full input: {}", input);
    lexer::lex(input);
}
