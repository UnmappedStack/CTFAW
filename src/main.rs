mod lexer;
mod parser;
mod statements;
mod ast;
mod error;

fn main() {
    let input: &str = "func fnName(arg1: u32, arg2: f64) -> u64 {let var: u64 = 21;doStuff(var);}"; // just as a test
    println!("Full input: {}", input);

    let tokens = lexer::lex(input);
    parser::parse(tokens);
}
