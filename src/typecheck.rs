/* Goes through the program and, as the file name suggests, checks the types. */

#![allow(unused_variables, unused_imports)]

use crate::backend::*;
use std::collections::HashMap;
use crate::parser::*;
use crate::lexer::*;
use crate::error::*;
use crate::ast::*;
use crate::statements::*;

pub fn typecheck_expr(mut expr: BranchChild, vars: &HashMap<String, Type>, program: &HashMap<String, FuncTableVal>) -> Type {
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
            match v.op {
                Operation::Less => return Type { val: TypeVal::Boolean, ptr_depth: 0 },
                Operation::LessEqu => return Type { val: TypeVal::Boolean, ptr_depth: 0 },
                Operation::Greater => return Type { val: TypeVal::Boolean, ptr_depth: 0 },
                Operation::GreaterEqu => return Type { val: TypeVal::Boolean, ptr_depth: 0 },
                Operation::Equ => return Type { val: TypeVal::Boolean, ptr_depth: 0 },
                Operation::NotEqu => return Type { val: TypeVal::Boolean, ptr_depth: 0 },
                _ => {},
            }
            match left.val {
                TypeVal::Any => right,
                _ => left,
            }
        },
        BranchChildVal::StrLit(_) => Type {val: TypeVal::Char, ptr_depth: 1},
        BranchChildVal::Float(_) => Type {val: TypeVal::F64, ptr_depth: 0},
        BranchChildVal::Char(_) => Type {val: TypeVal::Char, ptr_depth: 0},
        BranchChildVal::Ident(ref mut s) | BranchChildVal::Ref(ref mut s) => {
            let ret_type = match vars.get(s.as_str()) {
                Some(v) => {
                    let mut vc = v.clone();
                    if let BranchChildVal::Ref(_) = expr.val {
                        vc.ptr_depth += 1;
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
        BranchChildVal::Deref(ref mut s) => {
            let mut typ = typecheck_expr(*s.clone(), vars, program);
            typ.ptr_depth -= 1;
            typ
        }
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

fn typecheck_function(i: usize, func: (&String, &FuncTableVal), program: &mut HashMap<String, FuncTableVal>, globals: &Vec<GlobalVar>, startwith: &HashMap<String, Type>) {
    let mut local_vars = HashMap::new();
    local_vars.extend(startwith.clone());
    for global in globals {
        local_vars.insert(global.identifier.clone(), global.typ.clone());
    }
    for arg in func.1.signature.args.clone() {
        local_vars.insert(arg.val, arg.arg_type);
    }
    let statements_wrapped = func.1.statements.clone();
    match statements_wrapped {
        None => return,
        Some(_) => {},
    };
    let statements = statements_wrapped.unwrap();
    for statement in statements {
        match statement {
            Statement::Extern(s) => {
                program.insert(s.identifier.clone(), s.val.clone());
            },
            Statement::If(s) => {
                let mut second = func.1.clone();
                second.statements = Some(s.body.clone());
                typecheck_function(i, (func.0, &second), program, globals, &local_vars);
            },
            Statement::While(s) => {
                let mut second = func.1.clone();
                second.statements = Some(s.body.clone());
                typecheck_function(i, (func.0, &second), program, globals, &local_vars);
            },
            Statement::Define(s) => {
                typecheck_simple(s.def_type.clone(), s.expr.clone(), &local_vars, false, program);
                local_vars.insert(s.identifier.clone(), s.def_type.clone());
            },
            Statement::Assign(s) => {
                let mut ret_type = match local_vars.get(s.identifier.as_str()) {
                    Some(v) => v.clone(),
                    None => {
                        report_err(Component::ANALYSIS, s.ident_tok.clone(), format!("Undefined variable: {}", s.identifier).as_str());
                        unreachable!();
                    }
                };
                if s.deref { ret_type.ptr_depth -= 1 }
                typecheck_simple(ret_type.clone(), s.expr.clone(), &local_vars, false, program);
                let mut s_copy = s.clone();
                s_copy.typ = ret_type.clone();
            },
            Statement::Return(s) => {
                typecheck_simple(func.1.signature.ret_type.clone(), s.clone(), &local_vars, true, program);
            },
            Statement::FuncCall(c) => {
                let func = match program.get(&c.fn_ident) {
                    Some(f) => f,
                    None => {
                        report_err(Component::ANALYSIS, Token {val: TokenVal::Endln, row: c.row, col: c.col}, format!("Undefined function: {}", c.fn_ident).as_str());
                        unreachable!();
                    }
                };
                match func.signature.varargs_idx {
                    Some(v) => {
                        assert_report(c.args.len() >= v as usize, Component::ANALYSIS, Token {val: TokenVal::Endln, row: c.row, col: c.col}, "Incorrect number of arguments given to function call (has var args)");
                    },
                    None => {
                        assert_report(c.args.len() == func.signature.args.len(), Component::ANALYSIS, Token {val: TokenVal::Endln, row: c.row, col: c.col}, "Incorrect number of arguments given to function call (no var args)");
                    }
                }
                for (i, arg) in c.args.clone().into_iter().enumerate() {
                    match func.signature.varargs_idx {
                        Some(v) => if i >= v as usize { break },
                        None => {}
                    };
                    let val_type = typecheck_expr(arg, &local_vars, program);
                    assert_report(!(val_type != func.signature.args[i].arg_type && val_type.val != TypeVal::Any), Component::ANALYSIS, Token {val: TokenVal::Endln, row: c.row, col: c.col}, format!("Argument {} of function call recieved is type {:?}, expected type {:?}", i, val_type, func.signature.args[i].arg_type).as_str());
                }
            }
            _ => {}
        }
    }
}

pub fn typecheck(program: &mut HashMap<String, FuncTableVal>, globals: &Vec<GlobalVar>, startwith: &HashMap<String, Type>) {
    for (i, entry) in program.clone().iter().enumerate() {
        typecheck_function(i, entry, program, globals, startwith);
    }
}
