/* Parses the list of tokens to parse a statement list. Also makes use of the AST generator in `ast.rs` */

#![allow(dead_code, unused_variables)]

use crate::lexer::*;
use crate::ast::*;

// Some structures first need to be defined
// TODO: Add a generic assign statement used for both assigning existing vars and defining new ones

#[derive(Debug)]
pub struct DefineStatement {
    is_const: bool,
    pub identifier: String,
    def_type: Token,
    pub expr: BranchChild,
}

#[derive(Debug)]
pub struct AssignStatement {
    pub identifier: String,
    pub expr: BranchChild,
}

#[derive(Debug, Clone)]
pub struct FuncCallStatement {
    pub fn_ident: String,
    pub args: Vec<BranchChild>,
}

#[derive(Debug, Clone)]
pub struct AsmIOEntry {
    pub register: String,
    pub identifier: String,
}

#[derive(Debug)]
pub struct InlineAsmStatement {
    pub asm: String,
    pub inputs: Vec<AsmIOEntry>,
    pub outputs: Vec<AsmIOEntry>,
    pub clobbers: Vec<String>,
}

#[derive(Debug)]
pub enum Statement {
    Define(DefineStatement),
    Assign(AssignStatement),
    FuncCall(FuncCallStatement),
    InlineAsm(InlineAsmStatement),
    NullStatement, // NOTE: for debugging only, don't use in the actual compiler!
}

fn token_is_type(token: Token) -> bool {
    token == Token::U8      ||
    token == Token::U16     ||
    token == Token::U32     ||
    token == Token::U64     ||
    token == Token::F64     ||
    token == Token::Boolean
}

pub fn parse_define_statement(tokens: Vec<Token>) -> Statement {
    let is_const = tokens[0] == Token::Const;
    let identifier = if let Token::Ident(val) = tokens[1].clone() {
        val
    } else {
        assert!(false, "unreachable");
        String::from("ctfaw_failure")
    };
    assert!(
        tokens[2] == Token::Colon && token_is_type(tokens[3].clone()) && tokens[4] == Token::Assign, 
        "Invalid syntax for definition statement."
    );
    let expr = parse_expression(tokens[5..tokens.len() - 1].to_vec());
    Statement::Define(
        DefineStatement {
            is_const: is_const,
            identifier,
            def_type: tokens[3].clone(),
            expr
        }
    )
}

fn parse_assign_statement(tokens: Vec<Token>) -> Statement {
    assert!(tokens[1] == Token::Assign, "Couldn't parse statement, expected = but it wasn't there.");
    let expr = parse_expression(tokens[2..tokens.len() - 1].to_vec());
    let identifier = if let Token::Ident(val) = tokens[0].clone() {
        val
    } else {
        assert!(false, "unreachable");
        String::from("ctfaw_failure")
    }; 
    Statement::Assign(
        AssignStatement {
            identifier,
            expr,
        }
    )
}

fn parse_expr_list(tokens: Vec<Token>) -> Vec<BranchChild> {
    let mut arg_tokens: Vec<Vec<Token>> = Vec::new();
    let mut arg_idx: i64 = -1;
    for tok in 0..tokens.len() {
        if tokens[tok] == Token::Comma || tok == 0 {
            arg_tokens.push(Vec::new());
            arg_idx += 1;
            if tokens[tok] == Token::Comma { continue; }
        }
        let token = tokens[tok].clone();
        arg_tokens[arg_idx as usize].push(token);
    }
    let mut args: Vec<BranchChild> = Vec::new();
    for arg in arg_tokens {
        args.push(parse_expression(arg));
    }
    args
}

/* A small helper function to find the nth instance of a Token in a Vec<Token> */
fn get_index(v: Vec<Token>, occurrence: usize, value: Token) -> Option<usize> {
    v.iter()
        .enumerate()
        .filter(|(_, &ref v) | *v == value)
        .map(|(i, _)| i)
        .nth(occurrence - 1)
}

/* asm(asm : reg | identifier, reg | identifier, reg : identifier : reg, reg, reg);
 *      ^              ^                                   ^               ^
 * asm source       inputs list                        outputs list     clobbered register list
 */ 
pub fn parse_inline_asm_statement(tokens: Vec<Token>) -> Statement {
    let asm = if let Token::Str(val) = tokens[2].clone() {
        val
    } else {
        assert!(false, "Invalid syntax for asm(), expected assembly source body after parenthesis.");
        String::from("ctfaw_failure")
    };
    // get inputs & outputs
    let first_colon_idx  = get_index(tokens.clone(), 1, Token::Colon).unwrap();
    let second_colon_idx = get_index(tokens.clone(), 2, Token::Colon).unwrap();
    let third_colon_idx  = get_index(tokens.clone(), 3, Token::Colon).unwrap();
    let closing_paren_idx = tokens.len() - 2;
    let input_tokens  = &tokens[first_colon_idx + 1..second_colon_idx];
    let output_tokens = &tokens[second_colon_idx + 1..third_colon_idx];
    let input_split: Vec<_> = input_tokens
        .split(|e| *e == Token::Comma)
        .filter(|v| !v.is_empty())
        .collect();
    let output_split: Vec<_> = output_tokens
        .split(|e| *e == Token::Comma)
        .filter(|v| !v.is_empty())
        .collect();
    
    let io_split = Vec::from([input_split.clone(), output_split.clone()]);

    let mut io: Vec<Vec<AsmIOEntry>> = Vec::from([Vec::from([]), Vec::from([])]);
    for t in 0..2 {
        for i in 0..input_split.len() {
            assert!(input_split[i][1] == Token::BitOr, "Expected | in inline assembly input/output between register name and identifier, got other value.");
            let register = if let Token::Str(val) = io_split[t][i][0].clone() {
                val
            } else {
                assert!(false, "Expected register name as string literal in input/output for inline assembly, got other value.");
                String::from("ctfaw_failure")
            };
            let identifier = if let Token::Ident(val) = io_split[t][i][2].clone() {
                val
            } else {
                assert!(false, "Expected identifier in input/output for inline assembly, got other value.");
                String::from("ctfaw_failure")
            };
            io[t].push(AsmIOEntry {
                register,
                identifier,
            });
        }
    }
    
    let clobber_tokens = &tokens[third_colon_idx + 1..closing_paren_idx];
    let clobber_split: Vec<_> = clobber_tokens
        .split(|e| *e == Token::Comma)
        .filter(|v| !v.is_empty())
        .collect();
    let mut clobbers = Vec::new();
    for i in 0..clobber_split.len() {
        let reg = if let Token::Str(val) = clobber_split[i][0].clone() {
            val
        } else {
            assert!(false, "String expected in clobbered register list for inline assembly, got other value.");
            String::from("ctfaw_failure")
        };
        clobbers.push(reg);
    }

    Statement::InlineAsm(
        InlineAsmStatement {
            asm,
            inputs: io[0].clone(),
            outputs: io[1].clone(),
            clobbers
        }
    )
}

pub fn parse_func_call_statement(tokens: Vec<Token>) -> Statement {
    let identifier = if let Token::Ident(val) = tokens[0].clone() {
        val
    } else {
        assert!(false, "unreachable");
        String::from("ctfaw_failure")
    };
    let args = parse_expr_list(tokens[2..tokens.len() - 2].to_vec());
    Statement::FuncCall(
        FuncCallStatement {
            fn_ident: identifier,
            args
        }
    )
}

pub fn parse_statement(tokens: Vec<Token>) -> Statement {
    // Try to work out which kind of statement it is
    let mut iter = tokens.iter();
    let first_token = iter.next().unwrap();
    let second_token = iter.next().unwrap();
    match first_token {
        Token::Const | Token::Let => parse_define_statement(tokens),
        Token::Ident(func_name) => {
            match second_token {
                Token::Assign => parse_assign_statement(tokens),
                Token::Lparen => {
                    if func_name == "asm" {
                        parse_inline_asm_statement(tokens)
                    } else {
                        parse_func_call_statement(tokens)
                    }
                },
                _ => {
                    assert!(false, "Unknown statement type, could not parse. Compilation failed.");
                    Statement::NullStatement
                },
            }
        },
        _ => {
            assert!(false, "Unknown statement type, could not parse. Compilation failed.");
            Statement::NullStatement
        },
    }
}

