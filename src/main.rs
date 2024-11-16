#![allow(unused_variables)]

use std::process::Command;
use std::fs;
use std::env;

mod lexer;
mod parser;
mod statements;
mod ast;
mod backend;
mod error;

fn main() {
    let input: &str = &fs::read_to_string(env::args().collect::<Vec<_>>()[1].clone()).expect("Couldn't read input file.");
    println!("Full input:\n{}", input);

    let tokens = lexer::lex(input);
    let ir = parser::parse(tokens);
    backend::compile(ir);

    Command::new("nasm")
        .args(["-felf64", "-o", "out.o", "out.asm"])
        .output()
        .expect("Failed to run assembler.");
    Command::new("ld")
        .args(["-o", "out", "out.o"])
        .output()
        .expect("Failed to run linker.");
}
