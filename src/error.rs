#![allow(dead_code)]

use std::fs::File;
use std::io::{self, BufRead};
use std::env;
use crate::lexer::*;
use std::fmt;
use std::process;

const GRN: &str = "\x1B[0;32m";
const CYN: &str = "\x1B[0;36m";
const BRED: &str = "\x1B[1;31m";
const NCL: &str = "\x1B[0m";

pub enum Component {
    LEXER,
    PARSER,
    CODEGEN
}

fn read_specific_line(file_path: &str, line_number: usize) -> io::Result<String> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);
    for (current_line, line) in reader.lines().enumerate() {
        if current_line == line_number - 1 {
            return line;
        }
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "Unreachable",
    ))
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Component::LEXER => write!(f, "lexer"),
            Component::PARSER => write!(f, "parser"),
            Component::CODEGEN => write!(f, "codegen"),
        }
    }
}

pub fn report_err(component: Component, token: Token, msg: &str) {
    let fname = env::var("CTFAW_SRC_FILENAME").unwrap();
    let line = read_specific_line(fname.as_str(), token.row as usize).unwrap();
    let num_row_digits = token.row.to_string().chars().count();
    let mut row_spaces= String::new();
    let mut line_spaces = String::new();
    for i in 0..num_row_digits { row_spaces.push_str(" "); }
    for i in 0..token.col { line_spaces.push_str(" "); }
    println!("{BRED}Error{NCL}: {}", msg);
    println!("{CYN} -->{NCL} {}:{}:{}", fname, token.row, token.col);
    println!("{CYN}{} |{NCL}", row_spaces);
    println!("{CYN}{} |{NCL} {}", token.row, line);
    println!("{CYN}{} |{NCL}{}{GRN}^ error here{NCL}", row_spaces, line_spaces);
    println!("Exiting due to {} error, could not build.", component);
    process::exit(0xDEAD);
}

pub fn assert_report(condition: bool, component: Component, token: Token, msg: &str) {
    if !condition {
        report_err(component, token, msg);
    }
}
