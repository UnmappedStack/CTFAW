mod lexer;
mod error;

fn main() {
    let input: &str = "211 + 12;"; // just as a test
    println!("Full input: {}", input);
    lexer::lex(input);
}
