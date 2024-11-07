/* The lexer/tokeniser/whatever-you-wanna-call-it for CTFAW. */

#![allow(dead_code, unused_assignments)]

use crate::error::*;

// TODO: Implement string literals
#[derive(Debug)]
enum Token {
    // Mathematical operators
    ADD, SUB, DIV, MUL, POW, LPAREN, RPAREN,
    
    // Bitwise operators
    BITAND, BITOR, BITNOT, LEFTSHIFT, RIGHTSHIFT,

    // Logical operators
    AND, OR, NOT, GREATER, LESS, GREATEREQU, LESSEQU, EQU,

    // Values
    IDENT(String), INT(u64), FLOAT(f64), TRUE, FALSE,

    // Some keywords
    LET, CONST, IF, ELSE, ELSEIF, FUNC, WHILE, RETURN,

    // Other
    REF, DEREF, LBRACE, RBRACE, ENDLN, ASSIGN
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

pub fn lex(txt: &str) {
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
            ' ' => { c += 1; continue },
            // easy ones first
            '+' => tokens.push(Token::ADD),
            '-' => tokens.push(Token::SUB),
            '/' => tokens.push(Token::DIV),
            '(' => tokens.push(Token::LPAREN),
            ')' => tokens.push(Token::RPAREN),
            '^' => tokens.push(Token::POW),
            '~' => tokens.push(Token::BITNOT),
            '!' => tokens.push(Token::NOT),
            ';' => tokens.push(Token::ENDLN),
            '{' => tokens.push(Token::LBRACE),
            '}' => tokens.push(Token::RBRACE),
            // some less easy ones
            '=' => {
                match next {
                    '=' => tokens.push(Token::EQU),
                    _ => tokens.push(Token::ASSIGN),
                }
            },
            '&' => {
                match next {
                    '&' => tokens.push(Token::AND),
                    _ => tokens.push(Token::BITAND),
                }
                iter.next();
            },
            '*' => {
                match next {
                    '*' => tokens.push(Token::POW),
                    _ => tokens.push(Token::MUL),
                }
                iter.next();
            },
            '|' => {
                match next {
                    '|' => tokens.push(Token::OR),
                    _ => tokens.push(Token::BITOR),
                }
                iter.next();
            },
            '>' => {
                match next {
                    '>' => tokens.push(Token::RIGHTSHIFT),
                    '=' => tokens.push(Token::GREATEREQU),
                    _ => tokens.push(Token::GREATER),
                }
                iter.next();
            },
            '<' => {
                match next {
                    '<' => tokens.push(Token::LEFTSHIFT),
                    '=' => tokens.push(Token::LESSEQU),
                    _ => tokens.push(Token::LESS),
                }
                iter.next();
            },
            // number literals (both floats and integers)
            '0'..='9' => {
                let mut this_char: char = current_char;
                let mut i = 0;
                let mut is_float: bool = false;
                let mut whole = &txt[c..];
                while is_num_digit(this_char) && whole.len() > 0 {
                    if !is_num_digit(*iter.peek().unwrap()) || whole.len() == 1 { break }
                    if this_char == '.' { is_float = true; }
                    this_char = iter.next().unwrap();
                    whole = &whole[1..];
                    i += 1;
                }
                let num_str = &txt[c..c + i + 1];
                if is_float {
                    tokens.push(Token::FLOAT(num_str.parse::<f64>().unwrap()));
                } else {
                    tokens.push(Token::INT(num_str.parse::<u64>().unwrap())); 
                }
                c += i;
            },
            // handle both identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut this_char: char = current_char;
                let mut i = 0;
                let mut whole = &txt[c..];
                while is_ident_char(this_char) && whole.len() > 0 {
                    if !is_ident_char(*iter.peek().unwrap()) || whole.len() == 1 { break }
                    this_char = iter.next().unwrap();
                    whole = &whole[1..];
                    i += 1;
                }
                let s = &txt[c..c + i + 1];
                c += i;
                match s {
                    "let" => tokens.push(Token::LET),
                    "true" => tokens.push(Token::TRUE),
                    "false" => tokens.push(Token::FALSE),
                    "const" => tokens.push(Token::CONST),
                    "if" => tokens.push(Token::IF),
                    "else" => tokens.push(Token::ELSE),
                    "elseif" => tokens.push(Token::ELSEIF),
                    "func" => tokens.push(Token::FUNC),
                    "while" => tokens.push(Token::WHILE),
                    "return" => tokens.push(Token::RETURN),
                    _ => tokens.push(Token::IDENT(String::from(s))),
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
}


