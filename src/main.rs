#![allow(unused_variables)]

mod lexer;
mod parser;
mod statements;
mod ast;
mod backend;
mod error;

fn main() {
    let input: &str = "fnName(1, 2, 3, 4, 5, 6, 7);"; // just as a test
    println!("Full input: {}", input);

    let tokens = lexer::lex(input);
    assert!(false, "No tests to run.");
}
