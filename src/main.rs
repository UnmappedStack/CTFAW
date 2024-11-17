#![allow(unused_variables)]

use std::process::Command;
use std::fs;
use std::env;

mod lexer;
mod parser;
mod statements;
mod ast;
mod optimisation;
mod backend;
mod error;

fn main() {
    let input: &str = &fs::read_to_string(env::args().collect::<Vec<_>>()[1].clone()).expect("Couldn't read input file.");

    println!("[ SELF ] Compiling...");
    let tokens = lexer::lex(input);
    let mut global_vars = Vec::new();
    let ir = parser::parse(tokens, &mut global_vars);
    backend::compile(ir, global_vars);

    println!("[ NASM ] Assembling...");
    Command::new("nasm")
        .args(["-f", "elf64", "out.asm"])
        .status()
        .expect("Failed to run assembler");

    println!("[  LD  ] Linking...");
    Command::new("ld")
        .args(["-o", "out", "out.o"])
        .status()
        .expect("Failed to run linker");

    println!("[ SELF ] Built successfully, trying to run compiled program...");
    let output = Command::new("sh")
        .args(["-c", "./out ; echo Exited with status $?"])
        .output()
        .expect("Failed to run final program");
    println!("{}", String::from_utf8_lossy(&output.stdout));
}
