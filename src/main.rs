mod lexer;
mod parser;
mod error;

fn main() {
    let input: &str = "let val = \"this is a string\" + num + true; // This is a comment\nlet otherval = val + 1;"; // just as a test
    println!("Full input: {}", input);
    lexer::lex(input);
}
