/* The lexer/tokeniser/whatever-you-wanna-call-it for CTFAW. */

#![allow(dead_code, unused_assignments)]

use crate::error::*;

// TODO: Implement string literals
#[derive(Debug)]
enum TokenType {
    // Mathematical operators
    ADD, SUB, DIV, MUL, POW, LPAREN, RPAREN,
    
    // Bitwise operators
    BITAND, BITOR, BITNOT, LEFTSHIFT, RIGHTSHIFT,

    // Logical operators
    AND, OR, NOT, GREATER, LESS, GREATEREQU, LESSEQU, EQU,

    // Values
    IDENT, INT, FLOAT,

    // Some keywords
    LET, CONST, IF, ELSE, ELSEIF, FUNC, WHILE, RETURN,

    // Other
    REF, DEREF, LBRACE, RBRACE, ENDLN, ASSIGN
}

#[derive(Debug)]
enum TokenValue {
    Int(u64),
    Float(f64),
    String(String),
    NOVAL
}

#[derive(Debug)]
struct Token {
    ttype: TokenType,
    val: TokenValue,
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
            '+' => tokens.push(Token { ttype: TokenType::ADD, val: TokenValue::NOVAL}),
            '-' => tokens.push(Token { ttype: TokenType::SUB, val: TokenValue::NOVAL }),
            '/' => tokens.push(Token { ttype: TokenType::DIV, val: TokenValue::NOVAL }),
            '(' => tokens.push(Token { ttype: TokenType::LPAREN, val: TokenValue::NOVAL }),
            ')' => tokens.push(Token { ttype: TokenType::RPAREN, val: TokenValue::NOVAL }),
            '^' => tokens.push(Token { ttype: TokenType::POW, val: TokenValue::NOVAL }),
            '~' => tokens.push(Token { ttype: TokenType::BITNOT, val: TokenValue::NOVAL }),
            '!' => tokens.push(Token { ttype: TokenType::NOT, val: TokenValue::NOVAL }),
            ';' => tokens.push(Token { ttype: TokenType::ENDLN, val: TokenValue::NOVAL }),
            '{' => tokens.push(Token { ttype: TokenType::LBRACE, val: TokenValue::NOVAL }),
            '}' => tokens.push(Token { ttype: TokenType::RBRACE, val: TokenValue::NOVAL }),
            // some less easy ones
            '=' => {
                match next {
                    '=' => tokens.push(Token { ttype: TokenType::EQU, val: TokenValue::NOVAL }),
                    _ => tokens.push(Token { ttype: TokenType::ASSIGN, val: TokenValue::NOVAL }),
                }
            },
            '&' => {
                match next {
                    '&' => tokens.push(Token { ttype: TokenType::AND, val: TokenValue::NOVAL }),
                    _ => tokens.push(Token { ttype: TokenType::BITAND, val: TokenValue::NOVAL }),
                }
                iter.next();
            },
            '*' => {
                match next {
                    '*' => tokens.push(Token { ttype: TokenType::POW, val: TokenValue::NOVAL }),
                    _ => tokens.push(Token { ttype: TokenType::MUL, val: TokenValue::NOVAL }),
                }
                iter.next();
            },
            '|' => {
                match next {
                    '|' => tokens.push(Token { ttype: TokenType::OR, val: TokenValue::NOVAL }),
                    _ => tokens.push(Token { ttype: TokenType::BITOR, val: TokenValue::NOVAL }),
                }
                iter.next();
            },
            '>' => {
                match next {
                    '>' => tokens.push(Token { ttype: TokenType::RIGHTSHIFT, val: TokenValue::NOVAL }),
                    '=' => tokens.push(Token { ttype: TokenType::GREATEREQU, val: TokenValue::NOVAL }),
                    _ => tokens.push(Token { ttype: TokenType::GREATER, val: TokenValue::NOVAL }),
                }
                iter.next();
            },
            '<' => {
                match next {
                    '<' => tokens.push(Token { ttype: TokenType::LEFTSHIFT, val: TokenValue::NOVAL }),
                    '=' => tokens.push(Token { ttype: TokenType::LESSEQU, val: TokenValue::NOVAL }),
                    _ => tokens.push(Token { ttype: TokenType::LESS, val: TokenValue::NOVAL }),
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
                    tokens.push(Token { ttype: TokenType::FLOAT, val: TokenValue::Float(num_str.parse::<f64>().unwrap()) });
                } else {
                    tokens.push(Token { ttype: TokenType::INT, val: TokenValue::Int(num_str.parse::<u64>().unwrap()) }); 
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
                    "let" => tokens.push(Token { ttype: TokenType::LET, val: TokenValue::NOVAL }),
                    "const" => tokens.push(Token { ttype: TokenType::CONST, val: TokenValue::NOVAL }),
                    "if" => tokens.push(Token { ttype: TokenType::IF, val: TokenValue::NOVAL }),
                    "else" => tokens.push(Token { ttype: TokenType::ELSE, val: TokenValue::NOVAL }),
                    "elseif" => tokens.push(Token { ttype: TokenType::ELSEIF, val: TokenValue::NOVAL }),
                    "func" => tokens.push(Token { ttype: TokenType::FUNC, val: TokenValue::NOVAL }),
                    "while" => tokens.push(Token { ttype: TokenType::WHILE, val: TokenValue::NOVAL }),
                    "return" => tokens.push(Token { ttype: TokenType::RETURN, val: TokenValue::NOVAL }),
                    _ => tokens.push(Token { ttype: TokenType::IDENT, val: TokenValue::String(String::from(s)) }),
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


