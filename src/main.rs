mod lexer;
mod error;

fn main() {
    let input: &str = "let test = 23 * 4 / num;"; // just as a test
    lexer::lex(input);
}
