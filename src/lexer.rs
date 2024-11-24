/* The lexer/tokeniser/whatever-you-wanna-call-it for CTFAW. */

#![allow(dead_code, unused_assignments)]

use crate::error::*;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum Operation {
    Add, Sub, Div, Pow, Star,
}

#[derive(Debug, PartialEq, Clone)]
pub enum LitVal {
    Ident(String),
    Int(u64),
    Float(f64),
    Bool(u8),
    Str(String),
}

/* NOTE: The Any type is used only internally within the compiler and should *not* be possible to
 * use by the user within the language. */

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Any,
    U8,
    U16,
    U32,
    U64,
    F64,
    Boolean,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Literal {
    pub val: LitVal,
    pub typ: Type,
}

// TODO: Add signed integer types
#[derive(Debug, PartialEq, Clone)]
pub enum TokenVal {
    // Mathematical operators
    Ops(Operation), Lparen, Rparen, Arrow,
    
    // Bitwise operators
    BitOr, BitNot, LeftShift, RightShift,

    // Logical operators
    And, Or, Not, Greater, Less, GreaterEqu, LessEqu, Equ,

    Literal(Literal),
    Type(Type),

    // Some other keywords
    Let, Const, If, Else, ElseIf, Func, While, Return,

    // Other
    Ampersand, Comma, Colon, Lbrace, Rbrace, Endln, Assign
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub val: TokenVal,
    pub row: u64,
    pub col: u64,
}

pub fn is_val(tok: TokenVal) -> bool {
    match tok {
        TokenVal::Literal(_) => true,
        _ => false,
    }
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

fn parse_escape_characters(s: &mut String) {
    *s = s.replace("\\n", "\n");
    *s = s.replace("\\\"", "\"");
    *s = s.replace("\\r", "\r");
    *s = s.replace("\\t", "\t");
    *s = s.replace("\\'", "\'");
}

impl Token {
    fn new(token: TokenVal, row: usize, col: usize) -> Self {
        Self {
            val: token,
            row: row.try_into().unwrap(),
            col: col.try_into().unwrap(),
        }
    }
}

pub fn lex(txt: &str) -> Vec<Token> {
    let mut iter = txt.chars().peekable();
    let mut tokens = Vec::new();
    let mut c = 0;
    let mut row: usize = 1;
    let mut col: usize = 1;
    while let Some(current_char) = iter.next() {
        let mut next: char = ' ';
        match iter.peek() {
            Some(ch) => next = *ch,
            None => next = ' '
        }
        match current_char {
            ' ' | '\t' | '\r' => { c += 1; col += 1; continue },
            '\n' => { c += 1; col = 0; row += 1; continue },
            // easy ones first
            ':' => tokens.push(Token::new(TokenVal::Colon, row, col)),
            ',' => tokens.push(Token::new(TokenVal::Comma, row, col)),
            '+' => tokens.push(Token::new(TokenVal::Ops(Operation::Add), row, col)),
            '(' => tokens.push(Token::new(TokenVal::Lparen, row, col)),
            ')' => tokens.push(Token::new(TokenVal::Rparen, row, col)),
            '^' => tokens.push(Token::new(TokenVal::Ops(Operation::Pow), row, col)),
            '~' => tokens.push(Token::new(TokenVal::BitNot, row, col)),
            '!' => tokens.push(Token::new(TokenVal::Not, row, col)),
            ';' => tokens.push(Token::new(TokenVal::Endln, row, col)),
            '{' => tokens.push(Token::new(TokenVal::Lbrace, row, col)),
            '}' => tokens.push(Token::new(TokenVal::Rbrace, row, col)),
            // some less easy ones
            '-' => {
                match next {
                    '>' => {tokens.push(Token::new(TokenVal::Arrow, row, col)); iter.next(); c += 1; col += 1; }
                    _ => tokens.push(Token::new(TokenVal::Ops(Operation::Sub), row, col)),
                }
            },
            '=' => {
                match next {
                    '=' => {tokens.push(Token::new(TokenVal::Equ, row, col)); iter.next(); c += 1; col += 1; }
                    _ => tokens.push(Token::new(TokenVal::Assign, row, col)),
                }
            },
            '&' => {
                match next {
                    '&' => {tokens.push(Token::new(TokenVal::And, row, col)); iter.next(); c += 1; col += 1; }
                    _ => tokens.push(Token::new(TokenVal::Ampersand, row, col)), // could be deref *or* bitwise AND. That's for the parser to work out.
                }
            },
            '*' => {
                match next {
                    '*' => {tokens.push(Token::new(TokenVal::Ops(Operation::Pow), row, col)); iter.next(); c += 1; col += 1; }
                    _ => tokens.push(Token::new(TokenVal::Ops(Operation::Star), row, col)),
                }
            },
            '|' => {
                match next {
                    '|' => {tokens.push(Token::new(TokenVal::Or, row, col)); iter.next(); c += 1; col += 1; }
                    _ => tokens.push(Token::new(TokenVal::BitOr, row, col)),
                }
            },
            '>' => {
                match next {
                    '>' => {tokens.push(Token::new(TokenVal::RightShift, row, col)); iter.next(); c += 1; col += 1; }
                    '=' => {tokens.push(Token::new(TokenVal::GreaterEqu, row, col)); iter.next(); c += 1; col += 1; }
                    _ => tokens.push(Token::new(TokenVal::Greater, row, col)),
                }
            },
            '<' => {
                match next {
                    '<' => {tokens.push(Token::new(TokenVal::LeftShift, row, col)); iter.next(); c += 1; col += 1; }
                    '=' => {tokens.push(Token::new(TokenVal::LessEqu, row, col)); iter.next(); c += 1; col += 1; }
                    _ => tokens.push(Token::new(TokenVal::Less, row, col)),
                }
            },
            // comment and division symbol handling
            '/' => {
                match next {
                    '/' => {
                        let mut this_char: char = current_char;
                        let mut whole = &txt[c..];
                        while this_char != '\n' || whole.len() > 1 {
                            if *iter.peek().unwrap() == '\n' { c += 1; col += 1; break }
                            this_char = iter.next().unwrap();
                            whole = &whole[1..];
                            c += 1;
                            col += 1;
                        }
                        iter.next();
                    },
                    _ => tokens.push(Token::new(TokenVal::Ops(Operation::Div), row, col)),
                }
            }
            // number literals (both floats and integers)
            '0'..='9' => {
                let mut this_char: char = current_char;
                let mut i = 0;
                let mut is_float: bool = false;
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
                    tokens.push(
                        Token::new(
                            TokenVal::Literal(
                                Literal {
                                    val: LitVal::Float(num_str.parse::<f64>().unwrap()),
                                    typ: Type::F64
                                }
                            ),
                        row, col)
                    );
                } else {
                    tokens.push(
                        Token::new(
                            TokenVal::Literal(
                                Literal {
                                    val: LitVal::Int(num_str.parse::<u64>().unwrap()),
                                    typ: Type::Any,
                                }
                            )
                        , row, col)
                    );
                }
                c += i;
                col += i;
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
                let mut string = String::from(s);
                parse_escape_characters(&mut string);
                // TODO: Make string literals become a char* type instead of u64
                tokens.push(Token::new(TokenVal::Literal(Literal {val: LitVal::Str(string), typ: Type::U64}), row, col));
                c += i + 1;
                col += i + 1;
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
                col += i;
                match s {
                    "f64" => tokens.push(Token::new(TokenVal::Type(Type::F64), row, col)),
                    "u8" => tokens.push(Token::new(TokenVal::Type(Type::U8), row, col)),
                    "u16" => tokens.push(Token::new(TokenVal::Type(Type::U16), row, col)),
                    "u32" => tokens.push(Token::new(TokenVal::Type(Type::U32), row, col)),
                    "u64" => tokens.push(Token::new(TokenVal::Type(Type::U64), row, col)),
                    "bool" => tokens.push(Token::new(TokenVal::Type(Type::Boolean), row, col)),
                    "let" => tokens.push(Token::new(TokenVal::Let, row, col)),
                    "true" => tokens.push(Token::new(TokenVal::Literal(Literal {val: LitVal::Bool(1), typ: Type::Boolean}), row, col)),
                    "false" => tokens.push(Token::new(TokenVal::Literal(Literal {val: LitVal::Bool(0), typ: Type::Boolean}), row, col)),
                    "const" => tokens.push(Token::new(TokenVal::Const, row, col)),
                    "if" => tokens.push(Token::new(TokenVal::If, row, col)),
                    "else" => tokens.push(Token::new(TokenVal::Else, row, col)),
                    "elseif" => tokens.push(Token::new(TokenVal::ElseIf, row, col)),
                    "func" => tokens.push(Token::new(TokenVal::Func, row, col)),
                    "while" => tokens.push(Token::new(TokenVal::While, row, col)),
                    "return" => tokens.push(Token::new(TokenVal::Return, row, col)),
                    _ => tokens.push(Token::new(TokenVal::Literal(Literal { val: LitVal::Ident(String::from(s)), typ: Type::Any}), row, col)),
                }
            },
            _ => {
                report_err(Component::LEXER, Token::new(TokenVal::Endln, row, col), format!("Invalid symbol: \"{current_char}\"").as_str());
            },
        }
        c += 1;
        col += 1;
    }
    tokens
}


