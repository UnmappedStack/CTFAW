/* Parses the list of tokens to create a function table and statement list. Also makes use of the
 * AST generator in `ast.rs`
 */

#![allow(dead_code, unused_variables)]

use crate::lexer::*;
use crate::ast::*;

// Some structures first need to be defined

#[derive(Debug)]
struct DefineStatement {
    is_const: bool,
    identifier: String,
    def_type: Token,
    expr: BranchChild,
}

#[derive(Debug)]
struct AssignStatement {
    identifier: String,
    expr: BranchChild,
}

#[derive(Debug)]
struct FuncCallStatement {
    fn_ident: String,
    args: Vec<BranchChild>,
}

#[derive(Debug)]
struct AsmIOEntry {
    register: String,
    identifier: String,
}

#[derive(Debug)]
struct InlineAsmStatement {
    asm: String,
    inputs: Vec<AsmIOEntry>,
    outputs: Vec<AsmIOEntry>,
    clobbers: Vec<String>,
}

#[derive(Debug)]
enum Statement {
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

fn parse_define_statement(tokens: Vec<Token>) -> Statement {
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

fn parse_inline_asm_statement(tokens: Vec<Token>) -> Statement {
    println!("Inline asm statement.");
    Statement::NullStatement
}

fn parse_func_call_statement(tokens: Vec<Token>) -> Statement {
    println!("Function call statement.");
    Statement::NullStatement
}

pub fn parse_statement(tokens: Vec<Token>) {
    // Try to work out which kind of statement it is
    let mut iter = tokens.iter();
    let first_token = iter.next().unwrap();
    let second_token = iter.next().unwrap();
    let result = match first_token {
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
    };
    println!("Resulting statement: {:?}", result);
}











