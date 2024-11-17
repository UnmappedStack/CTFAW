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
    println!("Compiling...");
    let tokens = lexer::lex(input);
    let ir = parser::parse(tokens);
    backend::compile(ir);
    println!("Assembling...");
    Command::new("nasm")
        .args(["-f", "elf64", "out.asm"])
        .status()
        .expect("Failed to run assembler");
    println!("Linking...");
    Command::new("ld")
        .args(["-o", "out", "out.o"])
        .status()
        .expect("Failed to run linker");
    println!("Built successfully, trying to run compiled program...");
    let output = Command::new("sh")
        .args(["-c", "./out ; echo Exited with status $?"])
        .output()
        .expect("Failed to run final program");
    println!("{}", String::from_utf8_lossy(&output.stdout));
}
