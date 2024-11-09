/* The lexer/tokeniser/whatever-you-wanna-call-it for CTFAW. */

#![allow(dead_code, unused_assignments)]

use crate::error::*;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum Operation {
    Add, Sub, Div, Pow, Star,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Mathematical operators
    Ops(Operation), Lparen, Rparen,
    
    // Bitwise operators
    BitOr, BitNot, LeftShift, RightShift,

    // Logical operators
    And, Or, Not, Greater, Less, GreaterEqu, LessEqu, Equ,

    // Values
    Ident(String), Int(u64), Float(f64), Bool(u8), Str(String),

    // Some keywords
    Let, Const, If, Else, ElseIf, Func, While, Return,

    // Other
    Ampersand, Comma, Colon, Lbrace, Rbrace, Endln, Assign
}

fn is_num_digit(ch: char) -> bool {
    (ch >= '0' && ch <= '9') || ch == '.'
}

fn is_ident_char(ch: char) -> bool {
    (ch >= '0' && ch <= '9')
     || (ch >= 'a' && ch <= 'z')
     || (ch >= 'A' && ch <= 'Z')
     || ch == '_' 
}

pub fn lex(txt: &str) -> Vec<Token> {
    let mut iter = txt.chars().peekable();
    let mut tokens = Vec::new();
    let mut c = 0;
    while let Some(current_char) = iter.next() {
        let mut next: char = ' ';
        match iter.peek() {
            Some(ch) => next = *ch,
            None => next = ' '
        }
        match current_char {
            ' ' |  '\n' | '\t' | '\r' => { c += 1; continue },
            // easy ones first
            ':' => tokens.push(Token::Colon),
            ',' => tokens.push(Token::Comma),
            '+' => tokens.push(Token::Ops(Operation::Add)),
            '-' => tokens.push(Token::Ops(Operation::Sub)),
            '(' => tokens.push(Token::Lparen),
            ')' => tokens.push(Token::Rparen),
            '^' => tokens.push(Token::Ops(Operation::Pow)),
            '~' => tokens.push(Token::BitNot),
            '!' => tokens.push(Token::Not),
            ';' => tokens.push(Token::Endln),
            '{' => tokens.push(Token::Lbrace),
            '}' => tokens.push(Token::Rbrace),
            // some less easy ones
            '=' => {
                match next {
                    '=' => {tokens.push(Token::Equ); iter.next();},
                    _ => tokens.push(Token::Assign),
                }
            },
            '&' => {
                match next {
                    '&' => {tokens.push(Token::And); iter.next();},
                    _ => tokens.push(Token::Ampersand), // could be deref *or* bitwise AND. That's for the parser to work out.
                }
            },
            '*' => {
                match next {
                    '*' => {tokens.push(Token::Ops(Operation::Pow)); iter.next();},
                    _ => tokens.push(Token::Ops(Operation::Star)),
                }
            },
            '|' => {
                match next {
                    '|' => {tokens.push(Token::Or); iter.next();},
                    _ => tokens.push(Token::BitOr),
                }
            },
            '>' => {
                match next {
                    '>' => {tokens.push(Token::RightShift); iter.next();},
                    '=' => {tokens.push(Token::GreaterEqu); iter.next();},
                    _ => tokens.push(Token::Greater),
                }
            },
            '<' => {
                match next {
                    '<' => {tokens.push(Token::LeftShift); iter.next();},
                    '=' => {tokens.push(Token::LessEqu); iter.next();},
                    _ => tokens.push(Token::Less),
                }
            },
            // comment and division symbol handling
            '/' => {
                match next {
                    '/' => {
                        let mut this_char: char = current_char;
                        let mut whole = &txt[c..];
                        while this_char != '\n' || whole.len() > 1 {
                            if *iter.peek().unwrap() == '\n' { c += 1; break }
                            this_char = iter.next().unwrap();
                            whole = &whole[1..];
                            c += 1;
                        }
                        iter.next();
                    },
                    _ => tokens.push(Token::Ops(Operation::Div)),
                }
            }
            // number literals (both floats and integers)
            '0'..='9' => {
                let mut this_char: char = current_char;
                let mut i = 0;
                let mut is_float: bool = false;
                println!("num, c = {}", c);
                let mut whole = &txt[c..];
                while is_num_digit(this_char) && whole.len() > 1 {
                    if !is_num_digit(*iter.peek().unwrap()) { break }
                    if this_char == '.' { is_float = true; }
                    this_char = iter.next().unwrap();
                    whole = &whole[1..];
                    i += 1;
                }
                let num_str = &txt[c..c + i + 1];
                if is_float {
                    tokens.push(Token::Float(num_str.parse::<f64>().unwrap()));
                } else {
                    tokens.push(Token::Int(num_str.parse::<u64>().unwrap())); 
                }
                c += i;
            },
            // handle string literals
            '"' => {
                let mut this_char = *iter.peek().unwrap();
                let mut i = 0;
                let mut whole = &txt[c..];
                while (this_char != '"' ) && whole.len() > 1 {
                    if *iter.peek().unwrap() == '"' { break }
                    this_char = iter.next().unwrap();
                    whole = &whole[1..];
                    i += 1;
                }
                let s = &txt[c + 1..c + i + 1];
                tokens.push(Token::Str(String::from(s)));
                c += i + 1;
                iter.next();
            }
            // handle both identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut this_char: char = current_char;
                let mut i = 0;
                let mut whole = &txt[c..];
                while is_ident_char(this_char) && whole.len() > 1 {
                    if !is_ident_char(*iter.peek().unwrap()) { break }
                    this_char = iter.next().unwrap();
                    whole = &whole[1..];
                    i += 1;
                }
                let s = &txt[c..c + i + 1];
                c += i;
                match s {
                    "let" => tokens.push(Token::Let),
                    "true" => tokens.push(Token::Bool(1)),
                    "false" => tokens.push(Token::Bool(0)),
                    "const" => tokens.push(Token::Const),
                    "if" => tokens.push(Token::If),
                    "else" => tokens.push(Token::Else),
                    "elseif" => tokens.push(Token::ElseIf),
                    "func" => tokens.push(Token::Func),
                    "while" => tokens.push(Token::While),
                    "return" => tokens.push(Token::Return),
                    _ => tokens.push(Token::Ident(String::from(s))),
                }
            },
            _ => {
                println!("Found symbol: `{}`", current_char);
                report_err(Component::LEXER, "Invalid symbol.");
            },
        }
        c += 1;
    }
    println!("\r\nToken list: {:?}", tokens);
    tokens
}


