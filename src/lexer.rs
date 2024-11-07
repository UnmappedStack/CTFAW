/* The lexer/tokeniser/whatever-you-wanna-call-it for CTFAW. */

#![allow(dead_code, unused_assignments)]

use crate::error::*;

#[derive(Debug)]
enum TokenType {
    // Mathematical operators
    ADD, SUB, DIV, MUL, POW, LPAREN, RPAREN,
    
    // Bitwise operators
    BITAND, BITOR, BITNOT, LEFTSHIFT, RIGHTSHIFT,

    // Logical operators
    AND, OR, NOT, GREATER, LESS, GREATEREQU, LESSEQU,

    // Values
    IDENT, INT, FLOAT,

    // Some keywords
    LET, CONST, IF, ELSE, ELSEIF, FUNC, WHILE, RETURN,

    // Other
    REF, DEREF, LBRACE, RBRACE, ENDLN
}

#[derive(Debug)]
struct Token {
    ttype: TokenType,
    // Depending on the type, one of these may have a value
    ival: Option<u64>,
    fval: Option<f64>,
    // TODO: Implement string literals
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
            '+' => tokens.push(Token { ttype: TokenType::ADD, ival: None, fval: None }),
            '-' => tokens.push(Token { ttype: TokenType::SUB, ival: None, fval: None }),
            '/' => tokens.push(Token { ttype: TokenType::DIV, ival: None, fval: None }),
            '(' => tokens.push(Token { ttype: TokenType::LPAREN, ival: None, fval: None }),
            ')' => tokens.push(Token { ttype: TokenType::RPAREN, ival: None, fval: None }),
            '^' => tokens.push(Token { ttype: TokenType::POW, ival: None, fval: None }),
            '~' => tokens.push(Token { ttype: TokenType::BITNOT, ival: None, fval: None }),
            '!' => tokens.push(Token { ttype: TokenType::NOT, ival: None, fval: None }),
            ';' => tokens.push(Token { ttype: TokenType::ENDLN, ival: None, fval: None }),
            '{' => tokens.push(Token { ttype: TokenType::LBRACE, ival: None, fval: None }),
            '}' => tokens.push(Token { ttype: TokenType::RBRACE, ival: None, fval: None }),
            // some less easy ones
            '&' => {
                match next {
                    '&' => tokens.push(Token { ttype: TokenType::AND, ival: None, fval: None }),
                    _ => tokens.push(Token { ttype: TokenType::BITAND, ival: None, fval: None }),
                }
                iter.next();
            },
            '*' => {
                match next {
                    '*' => tokens.push(Token { ttype: TokenType::POW, ival: None, fval: None }),
                    _ => tokens.push(Token { ttype: TokenType::MUL, ival: None, fval: None }),
                }
                iter.next();
            },
            '|' => {
                match next {
                    '|' => tokens.push(Token { ttype: TokenType::OR, ival: None, fval: None }),
                    _ => tokens.push(Token { ttype: TokenType::BITOR, ival: None, fval: None }),
                }
                iter.next();
            },
            '>' => {
                match next {
                    '>' => tokens.push(Token { ttype: TokenType::RIGHTSHIFT, ival: None, fval: None }),
                    '=' => tokens.push(Token { ttype: TokenType::GREATEREQU, ival: None, fval: None }),
                    _ => tokens.push(Token { ttype: TokenType::GREATER, ival: None, fval: None }),
                }
                iter.next();
            },
            '<' => {
                match next {
                    '<' => tokens.push(Token { ttype: TokenType::LEFTSHIFT, ival: None, fval: None }),
                    '=' => tokens.push(Token { ttype: TokenType::LESSEQU, ival: None, fval: None }),
                    _ => tokens.push(Token { ttype: TokenType::LESS, ival: None, fval: None }),
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
                    tokens.push(Token { ttype: TokenType::FLOAT, ival: None, fval: Some(num_str.parse::<f64>().unwrap()) });
                } else {
                    tokens.push(Token { ttype: TokenType::INT, ival: Some(num_str.parse::<u64>().unwrap()), fval: None }); 
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
                    "let" => tokens.push(Token { ttype: TokenType::LET, ival: None, fval: None }),
                    "const" => tokens.push(Token { ttype: TokenType::CONST, ival: None, fval: None }),
                    "if" => tokens.push(Token { ttype: TokenType::IF, ival: None, fval: None }),
                    "else" => tokens.push(Token { ttype: TokenType::ELSE, ival: None, fval: None }),
                    "elseif" => tokens.push(Token { ttype: TokenType::ELSEIF, ival: None, fval: None }),
                    "func" => tokens.push(Token { ttype: TokenType::FUNC, ival: None, fval: None }),
                    "while" => tokens.push(Token { ttype: TokenType::WHILE, ival: None, fval: None }),
                    "return" => tokens.push(Token { ttype: TokenType::RETURN, ival: None, fval: None }),
                    _ => continue,
                }
            },
            _ => report_err(Component::LEXER, "Invalid symbol."),
        }
        c += 1;
    }
    println!("\r\nToken list: {:?}", tokens);
}


