#![allow(dead_code)]

use crate::ast::*;
use crate::lexer::*;

/* "Folds" an expression, meaning it evaluates an expression at compile time (if possible), so that
 * it doesn't have to evaluate every time the program is run. This is the absolute most basic form
 * of optimisation.
 */
fn fold_branch(branch: &mut BranchChild) -> bool {
    match branch {
        BranchChild::Branch(val) => {
            if !(fold_branch(val.left_val.as_mut()) && fold_branch(val.right_val.as_mut())) { return false }
            let left = match *val.left_val {
                BranchChild::Int(v) => v,
                _ => return false
            };
            let right = match *val.right_val {
                BranchChild::Int(v) => v,
                _ => return false
            };
            let result = match val.op {
                Operation::Add => {
                    left + right
                },
                Operation::Sub => {
                    left - right
                },
                Operation::Star => {
                    left * right
                },
                Operation::Div => {
                    left / right
                },
                _ => { return false }
            };
            *branch = BranchChild::Int(result);
            true
        },
        BranchChild::Int(val) => {
            true
        },
        _ => {
            false
        }
    }
}

pub fn fold_expr(ast: BranchChild) -> (bool, BranchChild) {
    let mut ast_clone = ast.clone();
    let can_fold = fold_branch(&mut ast_clone);
    if can_fold { (can_fold, ast_clone) } else { (can_fold, ast) }
}
