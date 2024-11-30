/* Parses the list of tokens to parse a statement list. Also makes use of the AST generator in `ast.rs` */

#![allow(dead_code, unused_variables)]

use crate::utils::*;
use crate::error::*;
use crate::lexer::*;
use crate::ast::*;

// Some structures first need to be defined
// TODO: Add a generic assign statement used for both assigning existing vars and defining new ones

#[derive(Debug, Clone)]
pub struct DefineStatement {
    is_const: bool,
    pub identifier: String,
    pub def_type: Type,
    pub type_tok: Token,
    pub expr: BranchChild,
}

#[derive(Debug, Clone)]
pub struct AssignStatement {
    pub deref: bool,
    pub typ: Type,
    pub identifier: String,
    pub ident_tok: Token,
    pub expr: BranchChild,
}

#[derive(Debug, Clone)]
pub struct FuncCallStatement {
    pub ident_tok: Token,
    pub fn_ident: String,
    pub args: Vec<BranchChild>,
    pub row: u64,
    pub col: u64,
}

#[derive(Debug, Clone)]
pub struct AsmIOEntry {
    pub register: String,
    pub identifier: String,
}

#[derive(Debug, Clone)]
pub struct InlineAsmStatement {
    pub asm: String,
    pub inputs: Vec<AsmIOEntry>,
    pub outputs: Vec<AsmIOEntry>,
    pub clobbers: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Define(DefineStatement),
    Assign(AssignStatement),
    FuncCall(FuncCallStatement),
    InlineAsm(InlineAsmStatement),
    Return(BranchChild),
    NullStatement, // NOTE: for debugging only, don't use in the actual compiler!
}

pub fn parse_define_statement(tokens: Vec<Token>) -> Statement {
    let is_const = tokens[0].val == TokenVal::Const;
    let identifier = get_ident(&tokens[1]);
    assert_report(
        tokens[2].val == TokenVal::Colon && token_is_type(tokens[3].val.clone()),
        Component::PARSER,
        tokens[2].clone(),
        "Invalid syntax for definition statement."
    );
    let typ = match tokens[3].val.clone() {
        TokenVal::Type(mut t) => {
            for tok in &tokens[4..] {
                if tok.val != TokenVal::Ops(Operation::Star) {break}
                t.ptr_depth += 1;
            }
            t
        },
        _ => {
            report_err(Component::PARSER, tokens[3].clone(), "Expected type in variable declaration.");
            unreachable!();
        }
    };
    let expr = parse_expression(tokens[(5 + typ.ptr_depth) as usize..tokens.len() - 1].to_vec());
    Statement::Define(
        DefineStatement {
            is_const: is_const,
            identifier,
            def_type: typ,
            type_tok: tokens[3].clone(),
            expr
        }
    )
}

fn parse_assign_statement(mut tokens: Vec<Token>, deref: bool) -> Statement {
    if tokens[0].val == TokenVal::Ops(Operation::Star) {
        tokens = (&tokens[1..]).to_vec();
    }
    assert_report(tokens[1].val == TokenVal::Assign, Component::PARSER, tokens[1].clone(), "Couldn't parse statement, expected = but it wasn't there.");
    let expr = parse_expression(tokens[2..tokens.len() - 1].to_vec());
    let identifier = get_ident(&tokens[0]);
    Statement::Assign(
        AssignStatement {
            deref,
            typ: Type {val: TypeVal::Any, ptr_depth: 0},
            identifier,
            ident_tok: tokens[0].clone(),
            expr,
        }
    )
}

fn parse_expr_list(tokens: Vec<Token>) -> Vec<BranchChild> {
    let mut arg_tokens: Vec<Vec<Token>> = Vec::new();
    let mut arg_idx: i64 = -1;
    for tok in 0..tokens.len() {
        if tokens[tok].val == TokenVal::Comma || tok == 0 {
            arg_tokens.push(Vec::new());
            arg_idx += 1;
            if tokens[tok].val == TokenVal::Comma { continue; }
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
fn get_index(v: Vec<Token>, occurrence: usize, value: TokenVal) -> Option<usize> {
    v.iter()
        .enumerate()
        .filter(|(_, &ref v) | v.val == value)
        .map(|(i, _)| i)
        .nth(occurrence - 1)
}

/* asm(asm : reg | identifier, reg | identifier, reg : identifier : reg, reg, reg);
 *      ^              ^                                   ^               ^
 * asm source       inputs list                        outputs list     clobbered register list
 */ 
pub fn parse_inline_asm_statement(tokens: Vec<Token>) -> Statement {
    let asm = get_str(&tokens[2]);
    // get inputs & outputs
    let first_colon_idx  = get_index(tokens.clone(), 1, TokenVal::Colon).unwrap();
    let second_colon_idx = get_index(tokens.clone(), 2, TokenVal::Colon).unwrap();
    let third_colon_idx  = get_index(tokens.clone(), 3, TokenVal::Colon).unwrap();
    let closing_paren_idx = tokens.len() - 2;
    let input_tokens  = &tokens[first_colon_idx + 1..second_colon_idx];
    let output_tokens = &tokens[second_colon_idx + 1..third_colon_idx];
    let input_split: Vec<_> = input_tokens
        .split(|e| e.val == TokenVal::Comma)
        .filter(|v| !v.is_empty())
        .collect();
    let output_split: Vec<_> = output_tokens
        .split(|e| e.val == TokenVal::Comma)
        .filter(|v| !v.is_empty())
        .collect();
    
    let io_split = Vec::from([input_split.clone(), output_split.clone()]);

    let mut io: Vec<Vec<AsmIOEntry>> = Vec::from([Vec::from([]), Vec::from([])]);
    for t in 0..2 {
        for i in 0..io_split[t].len() {
            assert_report(io_split[t][i][1].val == TokenVal::Ops(Operation::BitOr), Component::PARSER, io_split[t][i][1].clone(), "Expected | in inline assembly input/output between register name and identifier, got other value.");
            let register = get_str(&io_split[t][i][0]);
            let identifier = get_ident(&io_split[t][i][2]);
            io[t].push(AsmIOEntry {
                register,
                identifier,
            });
        }
    }
    
    let clobber_tokens = &tokens[third_colon_idx + 1..closing_paren_idx];
    let clobber_split: Vec<_> = clobber_tokens
        .split(|e| e.val == TokenVal::Comma)
        .filter(|v| !v.is_empty())
        .collect();
    let mut clobbers = Vec::new();
    for i in 0..clobber_split.len() {
        let reg = get_str(&clobber_split[i][0]);
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
    let identifier = get_ident(&tokens[0]);
    let args = parse_expr_list(tokens[2..tokens.len() - 2].to_vec());
    Statement::FuncCall(
        FuncCallStatement {
            ident_tok: tokens[0].clone(),
            fn_ident: identifier,
            args,
            row: tokens[0].row.clone(),
            col: tokens[0].col.clone(),
        }
    )
}

pub fn parse_return_statement(tokens: Vec<Token>) -> Statement {
    let expr_tokens = Vec::from(&tokens[1..tokens.len() - 1]);
    let expr: BranchChild = parse_expression(expr_tokens);
    Statement::Return(expr)
}

pub fn parse_statement(tokens: Vec<Token>) -> Statement {
    // Try to work out which kind of statement it is
    let mut iter = tokens.iter();
    let first_token = iter.next().unwrap();
    let second_token = iter.next().unwrap();
    let mut func_name_maybe = None;
    match &first_token.val {
        TokenVal::Return => parse_return_statement(tokens),
        TokenVal::Const | TokenVal::Let => parse_define_statement(tokens),
        TokenVal::Ops(v) => {
            if *v == Operation::Star {
                parse_assign_statement(tokens, true)
            } else {
                report_err(Component::PARSER, first_token.clone(), "Operation at the start of statement is not allowed.");
                Statement::NullStatement
            }
        }
        TokenVal::Literal(v) if {
            if let LitVal::Ident(s) = v.val.clone() {
                func_name_maybe = Some(s);
                true
            } else {
                false
            }
        } => {
            let func_name = func_name_maybe.unwrap();
            match &second_token.val {
                TokenVal::Assign => parse_assign_statement(tokens, false),
                TokenVal::Lparen => {
                    if func_name == "asm" {
                        parse_inline_asm_statement(tokens)
                    } else {
                        parse_func_call_statement(tokens)
                    }
                },
                _ => {
                    report_err(Component::PARSER, second_token.clone(), "Unknown statement type, could not parse. Compilation failed.");
                    Statement::NullStatement
                },
            }
        },
        _ => {
            report_err(Component::PARSER, first_token.clone(), "Unknown statement type, could not parse. Compilation failed.");
            Statement::NullStatement
        },
    }
}

