#![allow(dead_code, unused_variables)]

use crate::typecheck::*;
use std::io::Write as FileWrite;
use std::fs::File;
use std::collections::HashMap;
use crate::parser::*;
use crate::statements::*;
use crate::lexer::*;
use crate::ast::*;
use std::fmt::Write;
use crate::Flags;

// Registers in order of arguments for passing into a function with the SYS-V ABI
const REGS: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
const DEBUG: bool = true;

#[derive(Clone)]
pub struct CompiledAsm {
    text: String,
    data: String,
    rodata: String,
    externs: Vec<String>,
    globals: Vec<String>,
    string_literals: Vec<String>,
    num_strings: usize,
    spaces: String,
    num_subroutines: u64, // NOTE: This isn't referring to functions!
    flags: Flags,
}

#[derive(Clone, Debug)]
pub struct LocalVar {
    ident: String,
    typ: Type,
}

// Takes a type and outputs the size (in bytes)
fn type_to_size(typ: Type) -> u64 {
    if typ.ptr_depth > 0 { return 8 }
    match typ.val {
        TypeVal::U8 | TypeVal::I8 | TypeVal::Char | TypeVal::Boolean => 1,
        TypeVal::U16 | TypeVal::I16 => 2,
        TypeVal::U32 | TypeVal::I32 => 4,
        TypeVal::U64 | TypeVal::I64 | TypeVal::Any | TypeVal::F64 => 8,
    }
}

fn check_type_signed(typ: Type) -> bool {
    if typ.ptr_depth > 0 { return false }
    match typ.val {
        TypeVal::I8 | TypeVal::I16 | TypeVal::I32 | TypeVal::I64 | TypeVal::F64 => true,
        _ => false,
    }
}

// Takes a (64 bit) register and a type, outputs the corresponding register name of the right size
fn register_of_size(original: &str, typ: Type) -> String {
    if original.chars().nth(0).unwrap() == 'r' && original.chars().nth(1).unwrap().is_digit(10) {
        let mut copy = String::from(original);
        match type_to_size(typ) {
            1 => copy.push('b'),
            2 => copy.push('w'),
            4 => copy.push('d'),
            _ => {},
        };
        return copy
    }
    match type_to_size(typ) {
        1 => {
            let mut copy = String::from(&original[1..]);
            copy.replace_range(1..2, "l"); // copy[1] = 'l';
            if original == "rsi" || original == "rdi" {
                copy.insert(1, 'i');
            }
            copy
        },
        2 => {
            String::from(&original[1..])
        },
        4 => {
            let mut copy = String::from(original);
            copy.replace_range(0..1, "e"); // copy[0] = 'e';
            copy
        },
        8 => {
            String::from(original)
        },
        _ => unreachable!()
    }
}

fn ptr_ident_of_size(typ: Type) -> String {
    let s = match type_to_size(typ) {
        1 => "BYTE",
        2 => "WORD",
        4 => "DWORD",
        8 => "QWORD",
        _ => unreachable!()
    };
    String::from(s)
}

fn write_text(txt: &mut String, spaces: String, flags: Flags, new: &str) {
    if new.as_bytes()[0] == ';' as u8 && !flags.include_comments {return}
    let _ = txt.write_str(spaces.as_str());
    let replaced = new.replace("\n", format!("\n{spaces}").as_str());
    let _ = txt.write_str(replaced.as_str());
    let _ = txt.write_str("\n");
}

fn get_var_loc(v: String, locals: Vec<LocalVar>, globals: Vec<GlobalVar>, stack_args: Vec<LocalVar>) -> (String, Type) {
    let stack_args_pos = stack_args.iter().position(|s| s.ident == v);
    if let Some(val) = stack_args_pos {
        let mut off = 0;
        for a in stack_args.iter().rev() {
            if v == a.ident { break }
            off += 8;
        }
        let ptr_type = ptr_ident_of_size(stack_args[val].typ.clone());
        return (format!("{} [rsp + {}]", ptr_type, off), stack_args[val].typ.clone()) 
    }

    let local_pos = locals.iter().position(|s| s.ident == v);
    match local_pos {
        Some(val) => {
            let mut off = 0;
            for l in &locals {
                if v == l.ident { break }
                off += type_to_size(l.typ.clone());
            }
            let ptr_type = ptr_ident_of_size(locals[val].typ.clone());
            (format!("{} [rbp - {}]", ptr_type, off), locals[val].typ.clone())
        },
        None => {
            let global_pos = globals.iter().position(|s| s.identifier == v);
            match global_pos {
                Some(val) => (format!("{}", globals[val].val), globals[val].typ.clone()),
                None => { panic!("Variable not defined in current scope.") }
            }
        }
    }
}

/* Operands are in rax and rcx, and returns in rax. */
fn compile_operation(out: &mut CompiledAsm, op: Operation, rettype: Type) {
    let rax_sized = register_of_size("rax", rettype.clone());
    let rcx_sized = register_of_size("rcx", rettype.clone());
    let is_signed = check_type_signed(rettype.clone());
    match op {
        Operation::Mod => {
            let op = if is_signed { "idiv" } else { "div" };
            let rdx_sized = register_of_size("rdx", rettype.clone());
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("xchg {}, {}\n{} {}\nmov {}, {}", rax_sized, rcx_sized, op, rcx_sized, rax_sized, rdx_sized).as_str());
        },
        Operation::GreaterEqu => {
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "cmp rcx, rax");
            let op = if is_signed { "setge" } else { "setae" };
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("{} al", op).as_str());
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "and rax, 1");
        },
        Operation::LessEqu => {
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "cmp rcx, rax");
            let op = if is_signed { "setle" } else { "setbe" };
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("{} al", op).as_str());
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "and rax, 1");
        },
        Operation::Less => {
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "cmp rcx, rax");
            let op = if is_signed { "setl" } else { "setb" };
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("{} al", op).as_str());
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "and rax, 1");
        },
        Operation::Greater => {
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "cmp rcx, rax");
            let op = if is_signed { "setg" } else { "seta" };
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("{} al", op).as_str());
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "and rax, 1");
        },
        Operation::NotEqu => {
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "cmp rax, rcx");
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "setnz al");
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "and rax, 1");
        },
        Operation::Equ => {
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "cmp rax, rcx");
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "setz al");
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "and rax, 1");
        },
        Operation::And => {
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "and rax, rcx");
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "setnz al");
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "and rax, 1");
        },
        Operation::Or => {
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "or rax, rcx");
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "setnz al");
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "and rax, 1");
        },
        Operation::BitXor => {
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("xor {}, {}", rax_sized, rcx_sized).as_str());
        },
        Operation::Ampersand => {
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("and {}, {}", rax_sized, rcx_sized).as_str());
        },
        Operation::BitOr => {
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("or {}, {}", rax_sized, rcx_sized).as_str());
        },
        Operation::LeftShift => {
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("mov rcx, {}\nshl {}, cl", rcx_sized, rax_sized).as_str());
        },
        Operation::RightShift => {
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("mov rcx, {}\nshr {}, cl", rax_sized, rcx_sized).as_str());
        },
        Operation::Star => {
            let op = if is_signed { "imul" } else { "mul" };
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("{} {}", op, rcx_sized).as_str());
        },
        Operation::Add => {
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("add {}, {}", rax_sized, rcx_sized).as_str());
        },
        Operation::Sub => {
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("sub {}, {}", rcx_sized, rax_sized).as_str());
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("mov {}, {}", rax_sized, rcx_sized).as_str());
        },
        Operation::Div => {
            let op = if is_signed { "idiv" } else { "div" };
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("xchg {}, {}\n{} {}", rax_sized, rcx_sized, op, rcx_sized).as_str());
        },
        _ => {
            panic!("Unsupported operation.")
        }
    }
}

fn compile_union_operation(out: &mut CompiledAsm, program: &HashMap<String, FuncTableVal>, operation: UnaryOp, allvars: Vec<LocalVar>, globals: Vec<GlobalVar>, stack_args: Vec<LocalVar>, rettype: Type) {
    let rax_sized = register_of_size("rax", rettype.clone());
    match operation.op {
        Operation::BitNot => {
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("not {}", rax_sized).as_str());
        },
        Operation::Not => {
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "test rax, rax");
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "setnz al");
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "xor rax, 1");
        },
        _ => {
            panic!("Unary operator not implemented yet.");
        }
    }
}

/* The result of a single AST branch is stored in RAX. */
fn compile_ast_branch(out: &mut CompiledAsm, program: &HashMap<String, FuncTableVal>, branch: BranchChild, allvars: Vec<LocalVar>, globals: Vec<GlobalVar>, stack_args: Vec<LocalVar>, rettype: Type) {
    let rax_sized = register_of_size("rax", rettype.clone());
    match branch.val {
        BranchChildVal::Unary(val) => {
            compile_ast_branch(out, program, *val.val.clone(), allvars.clone(), globals.clone(), stack_args.clone(), rettype.clone());
            compile_union_operation(out, program, val, allvars, globals, stack_args, rettype.clone());
        }
        BranchChildVal::Branch(val) => {
            // compile it as a branch
            compile_ast_branch(out, program, *val.left_val, allvars.clone(), globals.clone(), stack_args.clone(), rettype.clone());
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "push rax");
            compile_ast_branch(out, program, *val.right_val, allvars, globals, stack_args, rettype.clone());
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "pop rcx");
            compile_operation(out, val.op, rettype);
        },
        BranchChildVal::Cast(val) => {
            let allvars_hash: HashMap<String, Type> = allvars.clone().into_iter()
                    .map(|var| (var.ident, var.typ))
                    .collect();
            let original_type = typecheck_expr(val.val.clone(), &allvars_hash, program);
            compile_ast_branch(out, program, val.val.clone(), allvars.clone(), globals.clone(), stack_args.clone(), original_type.clone());
            let original_size = type_to_size(original_type.clone());
            let new_size      = type_to_size(val.typ.clone());
            if (new_size > original_size) && (check_type_signed(original_type.clone()) && check_type_signed(val.typ.clone())) {
                let original_rax_sized = register_of_size("rax", original_type.clone());
                let rcx_sized = register_of_size("rcx", original_type);
                write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("mov {}, {}", rcx_sized, original_rax_sized).as_str());
                write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("movsx {}, {}", rax_sized, rcx_sized).as_str());

            }
        },
        BranchChildVal::Char(val) => {
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("mov {}, {}", rax_sized, val).as_str());
        },
        BranchChildVal::Int(val) => {
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("mov {}, {}", rax_sized, val).as_str());
        },
        BranchChildVal::Deref(val) => {
            let allvars_hash: HashMap<String, Type> = allvars.clone().into_iter()
                    .map(|var| (var.ident, var.typ))
                    .collect();
            compile_ast_branch(out, program, *val.clone(), allvars.clone(), globals.clone(), stack_args.clone(), typecheck_expr(*val.clone(), &allvars_hash, program));
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("mov {}, [{}]", rax_sized, rax_sized).as_str());
        },
        BranchChildVal::Ref(val) => {
            let loc = get_var_loc(val, allvars, globals, stack_args).0;
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("lea {}, {}", rax_sized, loc).as_str());
        },
        BranchChildVal::Ident(val) => {
            let loc = get_var_loc(val, allvars, globals, stack_args);
            let rax_sized = register_of_size("rax", loc.1);
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("mov {}, {}", rax_sized, loc.0).as_str());
        },
        BranchChildVal::Fn(val) => {
            compile_func_call(out, program, val, allvars, globals, stack_args);
        },
        BranchChildVal::StrLit(val) => {
            let mut stringchars: Vec<String> = val.chars().map(|c| (c as u8).to_string()).collect();
            stringchars.push(String::from("0")); // make sure it has a null terminator
            out.string_literals.push(stringchars.join(", "));
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("lea rax, [strlit{}]", out.num_strings).as_str());
            out.num_strings += 1;
        },
        _ => {
            panic!("Not implemented yet, note that expressions can't yet handle floating point values.")
        }
    }
}

pub fn compile_expression(out: &mut CompiledAsm, program: &HashMap<String, FuncTableVal>, ast: BranchChild, allvars: Vec<LocalVar>, globals: Vec<GlobalVar>, stack_args: Vec<LocalVar>, rettype: Type) {
    write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), ";; Solve expression");
    compile_ast_branch(out, program, ast, allvars, globals, stack_args, rettype);
}

pub fn compile_define(out: &mut CompiledAsm, program: &HashMap<String, FuncTableVal>, statement: DefineStatement, allvars: Vec<LocalVar>, globals: Vec<GlobalVar>, stack_args: Vec<LocalVar>) {
    //write_text(&mut out.data, format!("{}: dq 0", statement.identifier).as_str());
    let loc = get_var_loc(statement.identifier.clone(), allvars.clone(), globals.clone(), stack_args.clone());
    compile_expression(out, program, statement.expr, allvars.clone(), globals.clone(), stack_args.clone(), loc.1);
    write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!(";; Assign value to var {} and define it", statement.identifier).as_str());
    write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("mov {}, {}", loc.0, register_of_size("rax", statement.def_type)).as_str());

}

pub fn compile_assign(out: &mut CompiledAsm, program: &HashMap<String, FuncTableVal>, statement: AssignStatement, allvars: Vec<LocalVar>, globals: Vec<GlobalVar>, stack_args: Vec<LocalVar>) {
    let mut loc = get_var_loc(statement.identifier.clone(), allvars.clone(), globals.clone(), stack_args.clone());
    if statement.deref {loc.1.ptr_depth -= 1}
    compile_expression(out, program, statement.expr, allvars.clone(), globals.clone(), stack_args.clone(), loc.clone().1);
    if statement.deref {
        write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!(";; Assign value to var {}", statement.identifier).as_str());
        write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("mov rcx, {}", loc.0).as_str());
        write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("mov [rcx], {}", register_of_size("rax", loc.1)).as_str());
        return
    }
    write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("mov {}, {}", loc.0, register_of_size("rax", loc.1)).as_str());
}

pub fn compile_return(out: &mut CompiledAsm, program: &HashMap<String, FuncTableVal>, expr: BranchChild, allvars: Vec<LocalVar>, globals: Vec<GlobalVar>, stack_args: Vec<LocalVar>, func: FuncTableVal, num_reg_args: usize, stack_added: usize) {
    write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), ";; Early return from function");
    compile_expression(out, program, expr, allvars.clone(), globals.clone(), stack_args.clone(), func.signature.ret_type); // this already puts it into rax
    write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("add rsp, {}", stack_added).as_str());
    write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "pop rbp");
    write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "ret");
}

pub fn compile_inline_asm(out: &mut CompiledAsm, statement: InlineAsmStatement, allvars: Vec<LocalVar>, globals: Vec<GlobalVar>, stack_args: Vec<LocalVar>) {
    write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), ";; Inline assembly");
    for clobber in &statement.clobbers {
        write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("push {}", clobber).as_str());
    }
    for input in statement.inputs {
        write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("mov {}, {}", input.register, get_var_loc(input.identifier, allvars.clone(), globals.clone(), stack_args.clone()).0).as_str());
    }
    write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("{}", statement.asm).as_str());
    for output in statement.outputs {
        write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("mov {}, [{}]", get_var_loc(output.identifier, allvars.clone(), globals.clone(), stack_args.clone()).0, output.register).as_str());
    }
    for clobber in &statement.clobbers {
        write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("pop {}", clobber).as_str());
    }
}

pub fn compile_func_call(out: &mut CompiledAsm, program: &HashMap<String, FuncTableVal>, statement: FuncCallStatement, allvars: Vec<LocalVar>, globals: Vec<GlobalVar>, stack_args: Vec<LocalVar>) {
    for arg in 0..statement.args.len() {
        compile_expression(out, program, statement.args[arg].clone(), allvars.clone(), globals.clone(), stack_args.clone(), Type {val: TypeVal::U64, ptr_depth: 0});
        if arg < 6 {
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("mov {}, rax", REGS[arg]).as_str());
        } else {
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("push rax").as_str());
        }
    }
    write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("call {}", statement.fn_ident).as_str());
}

fn compile_if_statement(out: &mut CompiledAsm, program: &mut HashMap<String, FuncTableVal>, statement: IfStatement, all_vars: Vec<LocalVar>, globals: Vec<GlobalVar>, stack_args: Vec<LocalVar>, val: FuncTableVal, num_reg_args: usize, stack_added: usize) {
    compile_expression(out, program, statement.condition, all_vars.clone(), globals.clone(), stack_args.clone(), Type {val: TypeVal::Boolean, ptr_depth: 0});
    write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("cmp al, 0\nje sect{}", out.num_subroutines).as_str());
    compile_scope(out, program, all_vars, globals, stack_args, statement.body, val, num_reg_args, stack_added);
    write_text(&mut out.text, String::new(), out.flags.clone(), format!("sect{}:", out.num_subroutines).as_str());
    out.num_subroutines += 1;
}

fn compile_while_statement(out: &mut CompiledAsm, program: &mut HashMap<String, FuncTableVal>, statement: WhileStatement, all_vars: Vec<LocalVar>, globals: Vec<GlobalVar>, stack_args: Vec<LocalVar>, val: FuncTableVal, num_reg_args: usize, stack_added: usize) {
    write_text(&mut out.text, String::new(), out.flags.clone(), format!("sect{}:", out.num_subroutines).as_str());
    compile_expression(out, program, statement.condition, all_vars.clone(), globals.clone(), stack_args.clone(), Type {val: TypeVal::Boolean, ptr_depth: 0});
    write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("cmp al, 0\nje sect{}", out.num_subroutines + 1).as_str());
    compile_scope(out, program, all_vars, globals, stack_args, statement.body, val, num_reg_args, stack_added);
    write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("jmp sect{}", out.num_subroutines).as_str());
    out.num_subroutines += 1;
    write_text(&mut out.text, String::new(), out.flags.clone(), format!("sect{}:", out.num_subroutines).as_str());
    out.num_subroutines += 1;
}

// Returns whether or not there's an early return.
fn compile_scope(out: &mut CompiledAsm, functab: &mut HashMap<String, FuncTableVal>, all_vars: Vec<LocalVar>, globals: Vec<GlobalVar>, stack_args: Vec<LocalVar>, statements: Vec<Statement>, val: FuncTableVal, num_reg_args: usize, stack_added: usize) -> bool {
    let mut has_early_ret = false;
    for statement in statements {
        match statement {
            Statement::Assign(v) => { compile_assign(out, &functab, v, all_vars.clone(), globals.clone(), stack_args.clone()) },
            Statement::Define(v) => { compile_define(out, &functab, v, all_vars.clone(), globals.clone(), stack_args.clone()) },
            Statement::InlineAsm(v)=> { compile_inline_asm(out, v, all_vars.clone(), globals.clone(), stack_args.clone()) },
            Statement::FuncCall(v) => { compile_func_call(out, &functab, v, all_vars.clone(), globals.clone(), stack_args.clone()) },
            Statement::Return(v) => { compile_return(out, &functab, v, all_vars.clone(), globals.clone(), stack_args.clone(), val.clone(), num_reg_args.clone(), stack_added.clone()); has_early_ret = true; break },
            Statement::If(v) => { compile_if_statement(out, functab, v, all_vars.clone(), globals.clone(), stack_args.clone(), val.clone(), num_reg_args.clone(), stack_added.clone()) },
            Statement::While(v) => { compile_while_statement(out, functab, v, all_vars.clone(), globals.clone(), stack_args.clone(), val.clone(), num_reg_args.clone(), stack_added.clone()) },
            Statement::Extern(v) => {
                out.externs.push(v.identifier.clone()); functab.insert(v.identifier, v.val);
            },
            _ => { panic!("Cannot compile this statement") }
        }
    };
    has_early_ret
}

fn get_local_vars(statements: &Vec<Statement>) -> Vec<LocalVar> {
    let mut all_vars: Vec<LocalVar> = Vec::new();
    for statement in statements {
        match statement {
            Statement::Define(s) => {
                all_vars.push(LocalVar {
                    ident: s.identifier.clone(),
                    typ: s.def_type.clone(),
                });
            },
            Statement::If(s) => {
                all_vars.append(&mut get_local_vars(&s.body));
            },
            Statement::While(s) => {
                all_vars.append(&mut get_local_vars(&s.body));
            },
            _ => {},
        }
    }
    all_vars
}

pub fn compile(functab: &mut HashMap<String, FuncTableVal>, globals: Vec<GlobalVar>, mut externs: Vec<String>, flags: Flags) {
    let mut out = CompiledAsm { text: String::new(), data: String::new(), rodata: String::new(), externs: Vec::new(), globals: Vec::new(), string_literals: Vec::new(), num_strings: 0, spaces: String::new(), num_subroutines: 0, flags };
    for (key, val) in functab.clone().into_iter() {
        match val.statements.clone() {
            None => continue,
            _ => {},
        };
        out.globals.push(key.clone());
        out.spaces.clear();
        write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("\n{}: push rbp", key).as_str());
        for space in key.chars() {
            out.spaces.push_str(" ");
        }
        out.spaces.push_str("  ");
        let mut stack_args = Vec::new();
        let num_reg_args = 0; 
        // init local vars
        let mut all_vars = get_local_vars(&val.statements.clone().unwrap());
        let num_reg_args = if val.signature.args.len() <= 6 { val.signature.args.len() } else { 6 };
        let stack_added = (((all_vars.len() + num_reg_args) * 8) + 15) & !15;
        write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "mov rbp, rsp");
        write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("sub rsp, {}", stack_added).as_str());
        let mut reg_arg_off = 0;
        for (i, arg) in val.signature.args.iter().enumerate() {
            if i > 5 {
                stack_args.push(LocalVar {
                    ident: arg.val.clone(),
                    typ: arg.arg_type.clone(),
                });
                continue;
            }
            let sized_reg = register_of_size(REGS[i], arg.arg_type.clone());
            write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("mov {} [rbp - {}], {}", ptr_ident_of_size(arg.arg_type.clone()), reg_arg_off, sized_reg).as_str());
            all_vars.insert(i, LocalVar {
                ident: arg.val.clone(),
                typ: arg.arg_type.clone()
            });
            reg_arg_off += type_to_size(arg.arg_type.clone());
        }
        // now actually compile the statements
        if compile_scope(&mut out, functab, all_vars, globals.clone(), stack_args, val.statements.clone().unwrap().clone(), val.clone(), num_reg_args, stack_added) { continue }
        write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), format!("add rsp, {}", stack_added).as_str());
        write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "pop rbp");
        write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "xor rax, rax");
        write_text(&mut out.text, out.spaces.clone(), out.flags.clone(), "ret");
    }
    
    out.spaces.clear();

    for strlit in 0..out.num_strings {
        write_text(&mut out.rodata, out.spaces.clone(), out.flags.clone(), format!("strlit{}: db {}", strlit, out.string_literals[strlit]).as_str());
    }
    
    let mut file = File::create("out.asm").expect("Couldn't open file");
    let _ = file.write_all(format!("[BITS 64]\n\n").as_bytes());
    for global in out.globals {
        let _ = file.write_all(format!("global {}\n", global).as_bytes());
    }
    externs.extend(out.externs);
    for ext in externs {
        let _ = file.write_all(format!("extern {}\n", ext).as_bytes());
    }
    let _ = file.write_all(format!("\nsection .text\n{}\n", out.text).as_bytes());
    let _ = file.write_all(format!("section .data\n\n{}", out.data).as_bytes());
    let _ = file.write_all(format!("section .rodata\n\n{}", out.rodata).as_bytes());
}








