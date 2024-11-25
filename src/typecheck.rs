/* Goes through the program and, as the file name suggests, checks the types. */

#![allow(unused_variables, unused_imports)]

use std::collections::HashMap;
use crate::parser::*;
use crate::lexer::*;
use crate::error::*;
use crate::ast::*;
use crate::statements::*;

fn typecheck_expr(expr: BranchChild, vars: &HashMap<String, Type>) -> Type {
    match expr.val {
        BranchChildVal::Branch(v) => {
            let left = typecheck_expr(*v.left_val, vars);
            let right = typecheck_expr(*v.right_val, vars);
            if left != right &&
                    !(left == Type::Any || right == Type::Any) {
                report_err(Component::ANALYSIS, Token {val: TokenVal::Endln, row: expr.row, col: expr.col}, "Cannot operate on different types.");
                unreachable!();
            }
            match left {
                Type::Any => right,
                _ => left,
            }
        },
        BranchChildVal::StrLit(_) => Type::U64,
        BranchChildVal::Float(_) => Type::F64,
        BranchChildVal::Ident(s) | BranchChildVal::Ref(s) | BranchChildVal::Deref(s) => {
            let ret_type = match vars.get(s.as_str()) {
                Some(v) => v,
                None => {
                    report_err(Component::ANALYSIS, Token {val: TokenVal::Endln, row: expr.row, col: expr.col}, "Variable is not defined.");
                    unreachable!();
                }
            };
            ret_type.clone()
        },
        _ => Type::Any,
    }
}

fn typecheck_simple(ret_type: Type, expr: BranchChild, vars: &HashMap<String, Type>, is_ret_statement: bool) {
    let val_type = typecheck_expr(expr.clone(), vars);
    let error_message = if is_ret_statement {
        format!("Cannot return value of type {:?} from function of type {:?}", val_type, ret_type)
    } else {
        format!("Cannot assign value of type {:?} to variable of type {:?}", val_type, ret_type)
    };
    if val_type != ret_type && val_type != Type::Any {
        report_err(Component::ANALYSIS, Token {val: TokenVal::Endln, row: expr.row, col: expr.col}, error_message.as_str());
    }
}

pub fn typecheck(program: &HashMap<String, FuncTableVal>, globals: &Vec<GlobalVar>) {
    for (i, entry) in program.iter().enumerate() {
        let mut local_vars = HashMap::new();
        for global in globals {
            local_vars.insert(global.identifier.clone(), global.typ.clone());
        }
        let statements = &entry.1.statements;
        for statement in statements {
            match statement {
                Statement::Define(s) => {
                    typecheck_simple(s.def_type.clone(), s.expr.clone(), &local_vars, false);
                    local_vars.insert(s.identifier.clone(), s.def_type.clone());
                },
                Statement::Assign(s) => {
                    let ret_type = match local_vars.get(s.identifier.as_str()) {
                        Some(v) => v,
                        None => {
                            report_err(Component::ANALYSIS, s.ident_tok.clone(), format!("Undefined variable: {}", s.identifier).as_str());
                            unreachable!()
                        }
                    };
                    typecheck_simple(ret_type.clone(), s.expr.clone(), &local_vars, false);
                },
                Statement::Return(s) => {
                    typecheck_simple(entry.1.signature.ret_type.clone(), s.clone(), &local_vars, true);
                }
                _ => {}
            }
        }
    }
}
