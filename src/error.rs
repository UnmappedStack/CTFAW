#![allow(dead_code)]

use std::fmt;
use std::process;

pub enum Component {
    LEXER,
    PARSER,
    CODEGEN
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

pub fn report_err(component: Component, msg: &str) {
    println!("Error compiling in component {}: {}\r\nExiting, could not build.", component, msg);
    process::exit(0xDEAD);
}
