/* Part of the parser. Generates an AST for a token list representing an expression. */

#![allow(dead_code, unused_variables, unused_imports)]

use crate::parser::*;
use crate::error::*;
use std::collections::HashMap;
use crate::optimisation;
use crate::lexer::*;
use crate::statements::*;

#[derive(Debug, Clone)]
pub enum BranchChild {
    Branch(ASTBranch),
    Int(u64),
    Float(f64),
    Ident(String),
    StrLit(String),
    Ref(String),
    Deref(String),
    Fn(FuncCallStatement),
}

#[derive(Debug, Clone)]
pub struct ASTBranch {
    pub left_val: Box<BranchChild>,
    pub op: Operation,
    pub right_val: Box<BranchChild>,
}

/* Helper function to print X spaces */
fn print_level_spaces(depth: u32) {
    for _ in 0..depth * 2 {
        print!(" ");
    }
}

/* Helper function to print a single AST branch recursively (called by print_ast) */
fn print_ast_level(branch: &ASTBranch, depth: u32) {
    println!("Branch - {:?}:", branch.op);
    print_level_spaces(depth + 1);
    let left = &branch.left_val;
    match *((*left).clone()) {
        BranchChild::Branch(child_branch) => {
            print!("-> Left: ");
            print_ast_level(&child_branch, depth + 1);
        },
        _ => {
            println!("-> Left: Value: {:?}", left);
        }
    }
    let right = &branch.right_val;
    print_level_spaces(depth + 1);
    match *((*right).clone()) {
        BranchChild::Branch(child_branch) => {
            print!("-> Right: ");
            print_ast_level(&child_branch, depth + 1);
        },
        _ => {
            println!("-> Right: Value: {:?}", right);
        }
    }
}

/* Debug function to print an entire AST */
pub fn print_ast(root: &BranchChild) {
    print!("-> ");
    if let BranchChild::Branch(branch) = root {
        print_ast_level(branch, 0);
    } else {
        unreachable!();
    }
}

fn token_in_brackets(idx: u64, tokens: &[Token]) -> bool {
    let mut depth = 0;
    let mut i: u64 = 0;
    while i < tokens.len() as u64 {
        if idx == i {
            return depth != 0
        }
        match &tokens[i as usize].val {
            TokenVal::Lparen => { depth += 1; },
            TokenVal::Rparen => { depth -= 1; },
            _ => {},
        }
        i += 1;
    }
    panic!("idx > tokens.len() in token_in_brackets().");
}

/* Finds the index of the token with the highest priority.
 * If there are multiple of the same priority, it should pick the last.
 * It should always select a token *outside* of brackets, if there are any. 
 */
fn find_highest_priority_token(tokens: &mut &[Token], priorities: &HashMap<Operation, u8>) -> usize {
    // find the actual token
    let mut highest_priority_idx = 0;
    let mut max_priority = 0;
    let tokens_len = tokens.len();
    for idx in 0..tokens_len {
        if let TokenVal::Ops(op) = tokens[idx].val {
            let next = tokens[idx + 1].clone();
            if let TokenVal::Literal(l) = next.val {
                if let LitVal::Ident(v) = l.val {
                    if (idx == 0 || !is_val(tokens[idx - 1].val.clone())) &&
                        (tokens[idx].val == TokenVal::Ampersand || tokens[idx].val == TokenVal::Ops(Operation::Star)) { continue };
                }
            }
            if token_in_brackets(idx as u64, tokens) { continue };
            let priority = priorities[&op];
            if priority >= max_priority {
                max_priority = priority;
                highest_priority_idx = idx;
            }
        }
    }
    if max_priority == 0 {
        *tokens = &tokens[1..tokens.len() - 1];
        return find_highest_priority_token(tokens, priorities)
    }
    return highest_priority_idx
}

fn parse_branch(mut tokens: &[Token], priorities_map: &HashMap<Operation, u8>) -> Box<BranchChild> {
    let tokens_len = tokens.len();
    if tokens_len == 2 {
        let ident = get_ident(&tokens[1]);
        match tokens[0].val {
            TokenVal::Ampersand => return Box::new(BranchChild::Ref(ident)),
            TokenVal::Ops(Operation::Star) => return Box::new(BranchChild::Deref(ident)),
            _ => {
                report_err(Component::PARSER, tokens[0].clone(), "Unknown unary operation in expression.");
                return Box::new(BranchChild::Int(0));
            },
        }
    }
    
    let lit_o = if let TokenVal::Literal(v) = &tokens[0].val {Some(v)} else {None};

    if tokens_len == 1 {
        // It's a number so return a child with just a number
        match lit_o.expect("Value in expression which is not a number or identifier.").val.clone() {
            LitVal::Int(val) => return Box::new(BranchChild::Int(val)),
            LitVal::Float(val) => return Box::new(BranchChild::Float(val)),
            LitVal::Ident(val) => return Box::new(BranchChild::Ident(val.clone())),
            LitVal::Bool(val) => return Box::new(BranchChild::Int(val as u64)),
            LitVal::Str(val) => return Box::new(BranchChild::StrLit(val.clone())),
        }
    }

    match &tokens[0].val {
        TokenVal::Literal(Literal {val: LitVal::Ident(v), typ: _}) => {
            if (tokens[1].val == TokenVal::Lparen) && (tokens[tokens_len - 1].val == TokenVal::Rparen) {
                // All that's left is a function call statement. Parse it.
                let mut tokens_vec = Vec::from(tokens);
                tokens_vec.push(Token { val: TokenVal::Endln, row: 0, col: 0, } );
                let statement = parse_func_call_statement(tokens_vec);
                let fn_statement = if let Statement::FuncCall(val) = statement {
                    val
                } else {
                    unreachable!()
                };
                return Box::new(
                    BranchChild::Fn(
                        fn_statement
                    )
                )
            }
        },
        _ => {}
    }
    let max_priority_idx = find_highest_priority_token(&mut tokens, priorities_map);
    let left_branch = parse_branch(&tokens[..max_priority_idx], priorities_map);
    let right_branch = parse_branch(&tokens[max_priority_idx + 1..], priorities_map);
    Box::new(BranchChild::Branch(
        ASTBranch {
            left_val: left_branch,
            op: if let TokenVal::Ops(max_priority_token) = tokens[max_priority_idx].val {
                max_priority_token
            } else {
                unreachable!()
            },
            right_val: right_branch,
        }
    ))
}

/* Parses an expression into an AST.
 * Takes a list of tokens, all of which must be an operator, grouping symbol, number, or
 * identifier. Returns an ASTNode which is the root of an AST for this expression. */
pub fn parse_expression_full(tokens: Vec<Token>) -> (bool, BranchChild) {
    let priorities_map: HashMap<Operation, u8> = HashMap::from([
        (Operation::Pow, 1),
        (Operation::Star, 2),
        (Operation::Div, 2),
        (Operation::Sub, 3),
        (Operation::Add, 3),
    ]);
    optimisation::fold_expr(*parse_branch(&tokens[..], &priorities_map))
}

pub fn parse_expression(tokens: Vec<Token>) -> BranchChild {
    parse_expression_full(tokens).1
}


