#![allow(unused_variables)]

mod lexer;
mod parser;
mod statements;
mod ast;
mod backend;
mod error;

fn main() {
    let input: &str = "func fnName(num: u64, num2: u64) {let var: u64 = 12;}"; // just as a test
    println!("Full input: {}", input);

    let tokens = lexer::lex(input);
    let ir = parser::parse(tokens);
    backend::compile(ir);
}
