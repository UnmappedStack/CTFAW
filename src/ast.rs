/* Part of the parser. Generates an AST for a token list representing an expression. */

#![allow(dead_code, unused_variables)]

use std::collections::HashMap;
use crate::lexer::*;
use crate::statements::*;

#[derive(Debug, Clone)]
pub enum BranchChild {
    Branch(ASTBranch),
    Int(u64),
    Float(f64),
    Ident(String),
    Fn(FuncCallStatement),
}

#[derive(Debug, Clone)]
pub struct ASTBranch {
    left_val: Box<BranchChild>,
    op: Operation,
    right_val: Box<BranchChild>,
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
        assert!(false, "unreachable");
    }
}

/* Finds the index of the token with the highest priority.
 * If there are multiple of the same priority, it should pick the last.
 * It should always select a token *outside* of brackets, if there are any. 
 */
fn find_highest_priority_token(tokens: &[Token], priorities: &HashMap<Operation, u8>) -> usize {
    // find the indexes of the last and first parentheses & make sure they're matched
    let first_bracket = tokens.iter().position(|x| *x == Token::Lparen);
    let last_bracket = tokens.iter().rposition(|x| *x == Token::Rparen);
    let mut has_brackets = false;
    match first_bracket {
        Some(idx) => {
            has_brackets = true;
            if last_bracket == None {
                assert!(false, "Opening bracket ( is not matched.");
            }
        },
        _ => {
            if last_bracket != None {
                assert!(false, "Closing bracket ) is not matched.");
            }
        }
    }
    // find the actual token
    let mut highest_priority_idx = 0;
    let mut max_priority = 0;
    let tokens_len = tokens.len();
    for idx in 0..tokens_len {
        if let Token::Ops(op) = tokens[idx] {
            let priority = priorities[&op];
            if (priority >= max_priority) && !(has_brackets && (idx > first_bracket.unwrap() && idx < last_bracket.unwrap())) {
                max_priority = priority;
                highest_priority_idx = idx;
            }
        }
    }
    return highest_priority_idx
}

fn parse_branch(mut tokens: &[Token], priorities_map: &HashMap<Operation, u8>) -> Box<BranchChild> {
    let mut tokens_len = tokens.len();
    if tokens[0] == Token::Lparen && tokens[tokens_len - 1] == Token::Rparen {
        tokens = &tokens[1..tokens_len - 1];
        tokens_len -= 2;
    }
    if let Token::Ident(val) = &tokens[0] {
        if (tokens[1] == Token::Lparen) && (tokens[tokens_len - 1] == Token::Rparen) {
            // All that's left is a function call statement. Parse it.
            let mut tokens_vec = Vec::from(tokens);
            tokens_vec.push(Token::Endln);
            let statement = parse_func_call_statement(tokens_vec);
            let fn_statement = if let Statement::FuncCall(val) = statement {
                val
            } else {
                assert!(false, "Unreachable");
                FuncCallStatement { fn_ident: String::from("ctfaw_failure"), args: Vec::new() }
            };
            return Box::new(
                BranchChild::Fn(
                    fn_statement
                )
            )
        }
    }
    if tokens_len == 1 {
        // It's a number so return a child with just a number
        match &tokens[0] {
            Token::Int(val) => return Box::new(BranchChild::Int(*val)),
            Token::Float(val) => return Box::new(BranchChild::Float(*val)),
            Token::Ident(val) => return Box::new(BranchChild::Ident(val.clone())),
            Token::Bool(val) => return Box::new(BranchChild::Int(*val as u64)),
            _ => {
                assert!(false, "One symbol left in expression, not a number or identifier.");
                return Box::new(BranchChild::Int(0)); // this is just to make the compiler happy
            },
        }
    }
    let max_priority_idx = find_highest_priority_token(tokens, priorities_map);
    let left_branch = parse_branch(&tokens[..max_priority_idx], priorities_map);
    let right_branch = parse_branch(&tokens[max_priority_idx + 1..], priorities_map);
    Box::new(BranchChild::Branch(
        ASTBranch {
            left_val: left_branch,
            op: if let Token::Ops(max_priority_token) = tokens[max_priority_idx] {
                max_priority_token
            } else {
                assert!(false, "unreachable");
                Operation::Add // just to make the compiler happy 
            },
            right_val: right_branch,
        }
    ))
}

/* Parses an expression into an AST.
 * Takes a list of tokens, all of which must be an operator, grouping symbol, number, or
 * identifier. Returns an ASTNode which is the root of an AST for this expression. */
pub fn parse_expression(tokens: Vec<Token>) -> BranchChild {
    let priorities_map: HashMap<Operation, u8> = HashMap::from([
        (Operation::Pow, 1),
        (Operation::Star, 2),
        (Operation::Div, 2),
        (Operation::Sub, 3),
        (Operation::Add, 3),
    ]);
    *parse_branch(&tokens[..], &priorities_map)
}
