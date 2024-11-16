#![allow(unused_variables)]

mod lexer;
mod parser;
mod statements;
mod ast;
mod backend;
mod error;

fn main() {
    let input: &str = "let var: u64 = 23 * 4 - 3 / 4;"; // just as a test
    println!("Full input: {}", input);

    let tokens = lexer::lex(input);
    assert!(false, "No current tests.");
}
