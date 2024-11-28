/* Part of the parser. Generates an AST for a token list representing an expression. */

#![allow(dead_code, unused_variables, unused_imports)]

use crate::parser::*;
use crate::error::*;
use std::collections::HashMap;
use crate::optimisation;
use crate::lexer::*;
use crate::statements::*;


#[derive(Debug, Clone)]
pub struct Cast {
    pub val: BranchChild,
    pub typ: Type,
    pub original_type: Type,
}

#[derive(Debug, Clone)]
pub enum BranchChildVal {
    Branch(ASTBranch),
    Unary(UnaryOp),
    Char(u8),
    Int(u64),
    Float(f64),
    Ident(String),
    StrLit(String),
    Ref(String),
    Deref(Box<BranchChild>),
    Cast(Box<Cast>),
    Fn(FuncCallStatement),
}

#[derive(Debug, Clone)]
pub struct UnaryOp {
    pub op: Operation,
    pub val: Box<BranchChild>,
}

#[derive(Debug, Clone)]
pub struct BranchChild {
    pub val: BranchChildVal,
    pub row: u64,
    pub col: u64,
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
    match left.clone().val {
        BranchChildVal::Branch(child_branch) => {
            print!("-> Left: ");
            print_ast_level(&child_branch, depth + 1);
        },
        _ => {
            println!("-> Left: Value: {:?}", left);
        }
    }
    let right = &branch.right_val;
    print_level_spaces(depth + 1);
    match right.clone().val {
        BranchChildVal::Branch(child_branch) => {
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
    if let BranchChildVal::Branch(branch) = root.val.clone() {
        print_ast_level(&branch, 0);
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
    for idx in 0..tokens_len - 1 {
        if let TokenVal::Ops(op) = tokens[idx].val {
            let next = tokens[idx + 1].clone();
            if let TokenVal::Literal(l) = next.val {
                if let LitVal::Ident(v) = l.val {
                    if (idx == 0 || !is_val(tokens[idx - 1].val.clone())) &&
                        (tokens[idx].val == TokenVal::Ops(Operation::Ampersand) ||
                            tokens[idx].val == TokenVal::Ops(Operation::Star) ||
                            tokens[idx].val == TokenVal::Ops(Operation::Not) ||
                            tokens[idx].val == TokenVal::Ops(Operation::BitNot)) { continue };
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
    let num_lbraces = tokens.iter().filter(|&v| v.val == TokenVal::Lbrace).count();
    if tokens_len == 2 || tokens_len > 2 && (tokens[1].val == TokenVal::Lbrace && num_lbraces == 1) {
        match tokens[0].val {
            TokenVal::Ops(Operation::Ampersand) => return Box::new(BranchChild {val: BranchChildVal::Ref(get_ident(&tokens[1])), row: tokens[0].row, col: tokens[0].col}),
            TokenVal::Ops(Operation::Star) => return Box::new(BranchChild {val: BranchChildVal::Deref(parse_branch(&tokens[1..], priorities_map)), row: tokens[0].row, col: tokens[0].col}),
            TokenVal::Ops(Operation::BitNot) => return Box::new(BranchChild {val: BranchChildVal::Unary(UnaryOp {op: Operation::BitNot, val: parse_branch(&tokens[1..], priorities_map)}), row: tokens[0].row, col: tokens[0].col}),
            TokenVal::Ops(Operation::Not) => return Box::new(BranchChild {val: BranchChildVal::Unary(UnaryOp {op: Operation::Not, val: parse_branch(&tokens[1..], priorities_map)}), row: tokens[0].row, col: tokens[0].col}),
            _ => {
                report_err(Component::PARSER, tokens[0].clone(), "Unknown unary operation in expression.");
                return Box::new(BranchChild {val: BranchChildVal::Int(0), row: tokens[0].row, col: tokens[0].col});
            },
        }
    }
    
    let lit_o = if let TokenVal::Literal(v) = &tokens[0].val {Some(v)} else {None};

    if tokens_len == 1 {
        // It's a number so return a child with just a number
        match lit_o.expect("Value in expression which is not a number or identifier.").val.clone() {
            LitVal::Char(val) => return Box::new(BranchChild {val: BranchChildVal::Char(val), row: tokens[0].row, col: tokens[0].col}),
            LitVal::Int(val) => return Box::new(BranchChild {val: BranchChildVal::Int(val), row: tokens[0].row, col: tokens[0].col}),
            LitVal::Float(val) => return Box::new(BranchChild {val: BranchChildVal::Float(val), row: tokens[0].row, col: tokens[0].col}),
            LitVal::Ident(val) => return Box::new(BranchChild {val: BranchChildVal::Ident(val.clone()), row: tokens[0].row, col: tokens[0].col}),
            LitVal::Bool(val) => return Box::new(BranchChild {val: BranchChildVal::Int(val as u64), row: tokens[0].row, col: tokens[0].col}),
            LitVal::Str(val) => return Box::new(BranchChild {val: BranchChildVal::StrLit(val.clone()), row: tokens[0].row, col: tokens[0].col}),
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
                    BranchChild {
                        val: BranchChildVal::Fn(
                            fn_statement
                        ),
                        row: tokens[0].row,
                        col: tokens[0].col,
                    }
                )
            }
        },
        _ => {}
    }
    let max_priority_idx = find_highest_priority_token(&mut tokens, priorities_map);
    let max_priority_token = if let TokenVal::Ops(max_priority_token) = tokens[max_priority_idx].val {
        max_priority_token
    } else {
        unreachable!()
    };
    let left_branch = parse_branch(&tokens[..max_priority_idx], priorities_map);
    if max_priority_token == Operation::As {
        let result = Box::new(
            BranchChild {
                val: BranchChildVal::Cast(
                    Box::new(Cast {
                        val: *left_branch,
                        typ: {
                            let iter = &mut tokens[max_priority_idx + 1..].iter();
                            if let TokenVal::Type(mut t) = iter.next().unwrap().clone().val {
                                for tok in &tokens[max_priority_idx + 2..] {
                                    if tok.val != TokenVal::Ops(Operation::Star) {break}
                                    t.ptr_depth += 1;
                                }
                                t
                            } else {
                                report_err(Component::PARSER, tokens[2].clone(), "Expected type after `as` in cast, got something else.");
                                unreachable!();
                            }
                        },
                        original_type: Type {val: TypeVal::Any, ptr_depth: 0},
                    })
                ),
                row: tokens[max_priority_idx].row,
                col: tokens[max_priority_idx].col,
            }
        );
        return result
    }
    let right_branch = parse_branch(&tokens[max_priority_idx + 1..], priorities_map);
    Box::new(
        BranchChild {
            val: BranchChildVal::Branch(
                ASTBranch {
                    left_val: left_branch,
                    op: max_priority_token,
                    right_val: right_branch,
                }
            ),
            row: tokens[max_priority_idx].row,
            col: tokens[max_priority_idx].col,
        }
    )
}

/* Parses an expression into an AST.
 * Takes a list of tokens, all of which must be an operator, grouping symbol, number, or
 * identifier. Returns an ASTNode which is the root of an AST for this expression. */
pub fn parse_expression_full(tokens: Vec<Token>) -> (bool, BranchChild) {
    let priorities_map: HashMap<Operation, u8> = HashMap::from([
        (Operation::As, 1),
        (Operation::And, 2),
        (Operation::Or, 2),
        (Operation::Pow, 4),
        (Operation::Star, 5),
        (Operation::Div, 5),
        (Operation::Sub, 6),
        (Operation::Add, 6),
        (Operation::Ampersand, 7),
        (Operation::BitXor, 7),
        (Operation::BitOr, 7),
        (Operation::LeftShift, 8),
        (Operation::RightShift, 8),
    ]);
    optimisation::fold_expr(*parse_branch(&tokens[..], &priorities_map))
}

pub fn parse_expression(tokens: Vec<Token>) -> BranchChild {
    parse_expression_full(tokens).1
}


