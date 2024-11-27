/* Goes through the program and, as the file name suggests, checks the types. */

#![allow(unused_variables, unused_imports)]

use std::collections::HashMap;
use crate::parser::*;
use crate::lexer::*;
use crate::error::*;
use crate::ast::*;
use crate::statements::*;

fn typecheck_expr(mut expr: BranchChild, vars: &HashMap<String, Type>, program: &HashMap<String, FuncTableVal>) -> Type {
    match expr.val {
        BranchChildVal::Cast(v) => {
            typecheck_expr(v.val, vars, program);
            v.typ
        },
        BranchChildVal::Branch(v) => {
            let left = typecheck_expr(*v.left_val, vars, program);
            let right = typecheck_expr(*v.right_val, vars, program);
            if left != right &&
                    !(left.val == TypeVal::Any || right.val == TypeVal::Any) {
                report_err(Component::ANALYSIS, Token {val: TokenVal::Endln, row: expr.row, col: expr.col}, "Cannot operate on different types.");
                unreachable!();
            }
            match left.val {
                TypeVal::Any => right,
                _ => left,
            }
        },
        BranchChildVal::StrLit(_) => Type {val: TypeVal::Char, ptr_depth: 1},
        BranchChildVal::Float(_) => Type {val: TypeVal::F64, ptr_depth: 0},
        BranchChildVal::Char(_) => Type {val: TypeVal::Char, ptr_depth: 0},
        BranchChildVal::Ident(ref mut s) | BranchChildVal::Ref(ref mut s) | BranchChildVal::Deref(ref mut s) => {
            let ret_type = match vars.get(s.as_str()) {
                Some(v) => {
                    let mut vc = v.clone();
                    if let BranchChildVal::Ref(_) = expr.val {
                        vc.ptr_depth += 1;
                    } else if let BranchChildVal::Deref(_) = expr.val {
                        vc.ptr_depth -= 1;
                    }
                    vc
                },
                None => {
                    report_err(Component::ANALYSIS, Token {val: TokenVal::Endln, row: expr.row, col: expr.col}, "Variable is not defined.");
                    unreachable!();
                }
            };
            ret_type.clone()
        },
        BranchChildVal::Fn(f) => {
            match program.get(&f.fn_ident) {
                Some(func) => {
                    func.signature.ret_type.clone()
                },
                None => {
                    report_err(Component::ANALYSIS, f.ident_tok, "Function not defined.");
                    unreachable!();
                }
            }
        },
        _ => Type {val: TypeVal::Any, ptr_depth: 0},
    }
}

fn typecheck_simple(ret_type: Type, expr: BranchChild, vars: &HashMap<String, Type>, is_ret_statement: bool, program: &HashMap<String, FuncTableVal>) {
    let val_type = typecheck_expr(expr.clone(), vars, program);
    let error_message = if is_ret_statement {
        format!("Cannot return value of type {:?} from function of type {:?}", val_type, ret_type)
    } else {
        format!("Cannot assign value of type {:?} to variable of type {:?}", val_type, ret_type)
    };
    if val_type != ret_type && val_type.val != TypeVal::Any {
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
                    typecheck_simple(s.def_type.clone(), s.expr.clone(), &local_vars, false, program);
                    local_vars.insert(s.identifier.clone(), s.def_type.clone());
                },
                Statement::Assign(s) => {
                    let ret_type = match local_vars.get(s.identifier.as_str()) {
                        Some(v) => v,
                        None => {
                            report_err(Component::ANALYSIS, s.ident_tok.clone(), format!("Undefined variable: {}", s.identifier).as_str());
                            unreachable!();
                        }
                    };
                    typecheck_simple(ret_type.clone(), s.expr.clone(), &local_vars, false, program);
                    let mut s_copy = s.clone();
                    s_copy.typ = ret_type.clone();
                },
                Statement::Return(s) => {
                    typecheck_simple(entry.1.signature.ret_type.clone(), s.clone(), &local_vars, true, program);
                },
                Statement::FuncCall(c) => {
                    let func = match program.get(&c.fn_ident) {
                        Some(f) => f,
                        None => {
                            report_err(Component::ANALYSIS, Token {val: TokenVal::Endln, row: c.row, col: c.col}, format!("Undefined function: {}", c.fn_ident).as_str());
                            unreachable!();
                        }
                    };
                    assert_report(c.args.len() == func.signature.args.len(), Component::ANALYSIS, Token {val: TokenVal::Endln, row: c.row, col: c.col}, "Incorrect number of arguments given to function call.");
                    for (i, arg) in c.args.clone().into_iter().enumerate() {
                        let val_type = typecheck_expr(arg, &local_vars, program);
                        assert_report(val_type == func.signature.args[i].arg_type, Component::ANALYSIS, Token {val: TokenVal::Endln, row: c.row, col: c.col}, format!("Argument {} of function call recieved is type {:?}, expected type {:?}", i, val_type, func.signature.args[i].arg_type).as_str());
                    }
                }
                _ => {}
            }
        }
    }
}
