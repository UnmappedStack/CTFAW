#![allow(unused_variables)]

use std::process::Command;
use std::fs;
use std::env;

mod typecheck;
mod lexer;
mod parser;
mod statements;
mod ast;
mod optimisation;
mod backend;
mod error;

#[derive(Debug, Default)]
struct Flags {
    run: bool, // -r
    just_asm: bool, // -S
    just_obj: bool, // -c
    outfile_set: bool,
    out_file: String, // -o <filename>
}

fn check_flags_allowed(flags: &Flags) -> bool {
    if flags.run && (flags.just_asm || flags.just_obj) {
        println!("Cannot run program automatically if only generating object file or assembly file.");
        return false
    }
    if flags.just_asm && flags.just_obj {
        println!("Cannot have both -S and -c flags, must select one.");
        return false
    }
    true
}

fn help(arg0: &str) {
    println!("CTFAW Compiler, licensed under the Mozilla Public License 2.0 by Jake Steinburger (UnmappedStack).\n");
    println!("Usage:");
    println!("{} <input file path> -o <output file path> <options>\n", arg0);
    println!("Error: No input files to compile.");
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() == 1 {
        help(&args[0]);
        return
    }
    let mut input_files = Vec::new();
    let mut iter = args.iter().skip(1);
    let mut flags = Flags::default();
    while let Some(arg) = iter.next() {
        if arg.bytes().nth(0).unwrap() == '-' as u8 {
            match arg.as_str() {
                "-r" => flags.run = true,
                "-S" => flags.just_asm = true,
                "-c" => flags.just_obj = true,
                "-o" => {
                    flags.outfile_set = true;
                    flags.out_file = iter.next().expect("Expected filename after -o, got end of command.").to_string();
                },
                _ => {
                    println!("Unknown flag: {}\nCould not compile.", arg);
                }
            };
        } else {
            input_files.push(arg);
        }
    }
    if input_files.len() > 1 {
        println!("Currently CTFAW only supports passing a single input file. Compilation terminated.");
        return
    }
    env::set_var("CTFAW_SRC_FILENAME", input_files[0]);
    if !check_flags_allowed(&flags) { return }
    let input: &str = &fs::read_to_string(input_files[0]).expect("Couldn't read input file.");
    println!("[ SELF ] Compiling...");
    let tokens = lexer::lex(input);
    let mut global_vars = Vec::new();
    let ir = parser::parse(tokens, &mut global_vars);
    typecheck::typecheck(&ir, &global_vars);
    backend::compile(ir, global_vars);
    
    if flags.just_asm {
        if flags.outfile_set { let _ = fs::rename("out.asm", flags.out_file); }
        return
    }
    println!("[ NASM ] Assembling...");
    Command::new("nasm")
        .args(["-f", "elf64", "out.asm", "-g"])
        .status()
        .expect("Failed to run assembler");
    //let _ = fs::remove_file("out.asm");
    if flags.just_obj {
        if flags.outfile_set { let _ = fs::rename("out.o", flags.out_file); }
        return
    }
    println!("[  LD  ] Linking...");
    Command::new("ld")
        .args(["-o", "out", "out.o"])
        .status()
        .expect("Failed to run linker");
    let _ = fs::remove_file("out.o");
    if flags.outfile_set {
        let _ = fs::rename("out", flags.out_file.clone());
    } else {
        flags.out_file = String::from("out");
    }
    if !flags.run { return }
    println!("[ SELF ] Built successfully, trying to run compiled program...");
    let output = Command::new("sh")
        .args(["-c", format!("./{}; echo Exited with status $?", flags.out_file).as_str()])
        .output()
        .expect("Failed to run final program");
    println!("{}", String::from_utf8_lossy(&output.stdout));
}
