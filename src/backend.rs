#![allow(dead_code, unused_variables)]

use std::io::Write as FileWrite;
use std::fs::File;
use std::collections::HashMap;
use crate::parser::*;
use crate::statements::*;
use crate::lexer::*;
use crate::ast::*;
use std::fmt::Write;

// Registers in order of arguments for passing into a function with the SYS-V ABI
const REGS: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

#[derive(Clone)]
pub struct CompiledAsm {
    text: String,
    data: String,
    rodata: String,
    string_literals: Vec<String>,
    num_strings: usize,
    spaces: String,
}

fn write_text(txt: &mut String, spaces: String, new: &str) {
    let _ = txt.write_str(spaces.as_str());
    let replaced = new.replace("\n", format!("\n{spaces}").as_str());
    let _ = txt.write_str(replaced.as_str());
    let _ = txt.write_str("\n");
}

fn get_local_offset(v: String, allvars: Vec<String>) -> usize {
    match allvars.iter().position(|s| *s == v) {
        Some(val) => {
            val * 8
        },
        None => {
            panic!("Variable not defined in current scope.")
        }
    }
}

fn get_var_loc(v: String, locals: Vec<String>, globals: Vec<GlobalVar>) -> String {
    let local_pos = locals.iter().position(|s| *s == v);
    match local_pos {
        Some(val) => {
            format!("[rbp + {}]", val * 8)
        },
        None => {
            let global_pos = globals.iter().position(|s| s.identifier == v);
            match global_pos {
                Some(val) => format!("{}", globals[val].val),
                None => { panic!("Variable not defined in current scope.") }
            }
        }
    }
}

/* Operands are in rax and rbx, and returns in rax. */
fn compile_operation(out: &mut CompiledAsm, op: Operation) {
    match op {
        Operation::Star => {
            write_text(&mut out.text, out.spaces.clone(), "mul rbx");
        },
        Operation::Add => {
            write_text(&mut out.text, out.spaces.clone(), "add rax, rbx");
        },
        Operation::Sub => {
            write_text(&mut out.text, out.spaces.clone(), "sub rbx, rax");
            write_text(&mut out.text, out.spaces.clone(), "mov rax, rbx");
        },
        Operation::Div => {
            write_text(&mut out.text, out.spaces.clone(), "div rbx");
        },
        _ => {
            panic!("Unsupported operation.")
        }
    }
}

/* The result of a single AST branch is stored in RAX. */
fn compile_ast_branch(out: &mut CompiledAsm, branch: BranchChild, allvars: Vec<String>, globals: Vec<GlobalVar>) {
    match branch.val {
        BranchChildVal::Branch(val) => {
            // compile it as a branch
            compile_ast_branch(out, *val.left_val, allvars.clone(), globals.clone());
            write_text(&mut out.text, out.spaces.clone(), "push rax");
            compile_ast_branch(out, *val.right_val, allvars, globals);
            write_text(&mut out.text, out.spaces.clone(), "pop rbx");
            compile_operation(out, val.op);
        },
        BranchChildVal::Int(val) => {
            // just return the value
            write_text(&mut out.text, out.spaces.clone(), format!("mov rax, {}", val).as_str());
        },
        BranchChildVal::Deref(val) => {
            let loc = get_var_loc(val, allvars, globals);
            write_text(&mut out.text, out.spaces.clone(), format!("mov rax, [{}]\nmov rax, rax", loc).as_str());
            write_text(&mut out.text, out.spaces.clone(), format!("mov rax, rax").as_str());
        },
        BranchChildVal::Ref(val) => {
            let loc = get_var_loc(val, allvars, globals);
            write_text(&mut out.text, out.spaces.clone(), format!("lea rax, {}", loc).as_str());
        },
        BranchChildVal::Ident(val) => {
            let loc = get_var_loc(val, allvars, globals);
            write_text(&mut out.text, out.spaces.clone(), format!("mov rax, {}", loc).as_str());
        },
        BranchChildVal::Fn(val) => {
            compile_func_call(out, val, allvars, globals);
        },
        BranchChildVal::StrLit(val) => {
            let mut stringchars: Vec<String> = val.chars().map(|c| (c as u8).to_string()).collect();
            stringchars.push(String::from("0")); // make sure it has a null terminator
            out.string_literals.push(stringchars.join(", "));
            write_text(&mut out.text, out.spaces.clone(), format!("mov rax, strlit{}", out.num_strings).as_str());
            out.num_strings += 1;
        },
        _ => {
            panic!("Not implemented yet, expressions can't yet handle floating point values.")
        }
    }
}

pub fn compile_expression(out: &mut CompiledAsm, ast: BranchChild, allvars: Vec<String>, globals: Vec<GlobalVar>) {
    if let BranchChildVal::Branch(_) = ast.val {
        write_text(&mut out.text, out.spaces.clone(), "push rbx");
        compile_ast_branch(out, ast, allvars, globals);
        write_text(&mut out.text, out.spaces.clone(), "pop rbx");
    } else {
        compile_ast_branch(out, ast, allvars, globals);
    }
}

pub fn compile_define(out: &mut CompiledAsm, statement: DefineStatement, allvars: Vec<String>, globals: Vec<GlobalVar>) {
    //write_text(&mut out.data, format!("{}: dq 0", statement.identifier).as_str());
    compile_expression(out, statement.expr, allvars.clone(), globals.clone());
    let loc = get_var_loc(statement.identifier, allvars, globals);
    write_text(&mut out.text, out.spaces.clone(), format!("mov {}, rax", loc).as_str());

}

pub fn compile_assign(out: &mut CompiledAsm, statement: AssignStatement, allvars: Vec<String>, globals: Vec<GlobalVar>) {
    compile_expression(out, statement.expr, allvars.clone(), globals.clone());
    if statement.deref {
        write_text(&mut out.text, out.spaces.clone(), "push rbx");
        let loc = get_var_loc(statement.identifier, allvars, globals);
        write_text(&mut out.text, out.spaces.clone(), format!("mov rbx, {}", loc).as_str());
        write_text(&mut out.text, out.spaces.clone(), format!("mov [rbx], rax").as_str());
        write_text(&mut out.text, out.spaces.clone(), "pop rbx");
        return
    }
    let loc = get_var_loc(statement.identifier, allvars, globals);
    write_text(&mut out.text, out.spaces.clone(), format!("lea {}, [rax]", loc).as_str());
}

pub fn compile_return(out: &mut CompiledAsm, expr: BranchChild, allvars: Vec<String>, globals: Vec<GlobalVar>, func: FuncTableVal) {
    compile_expression(out, expr, allvars.clone(), globals.clone()); // this already puts it into rax
    for (i, v) in allvars.iter().enumerate() {
        write_text(&mut out.text, out.spaces.clone(), format!("pop rdi").as_str());
    }
    write_text(&mut out.text, out.spaces.clone(), "pop rbp");
    write_text(&mut out.text, out.spaces.clone(), "ret");
}

pub fn compile_inline_asm(out: &mut CompiledAsm, statement: InlineAsmStatement, allvars: Vec<String>, globals: Vec<GlobalVar>) {
    for clobber in &statement.clobbers {
        write_text(&mut out.text, out.spaces.clone(), format!("push {}", clobber).as_str());
    }
    for input in statement.inputs {
        write_text(&mut out.text, out.spaces.clone(), format!("mov {}, {}", input.register,  get_var_loc(input.identifier, allvars.clone(), globals.clone())).as_str());
    }
    write_text(&mut out.text, out.spaces.clone(), format!("{}", statement.asm).as_str());
    for output in statement.outputs {
        write_text(&mut out.text, out.spaces.clone(), format!("mov {}, [{}]", get_var_loc(output.identifier, allvars.clone(), globals.clone()), output.register).as_str());
    }
    for clobber in &statement.clobbers {
        write_text(&mut out.text, out.spaces.clone(), format!("pop {}", clobber).as_str());
    }
}

pub fn compile_func_call(out: &mut CompiledAsm, statement: FuncCallStatement, allvars: Vec<String>, globals: Vec<GlobalVar>) {
    for arg in 0..statement.args.len() {
        write_text(&mut out.text, out.spaces.clone(), "push rax");
        compile_expression(out, statement.args[arg].clone(), allvars.clone(), globals.clone());
        if arg < 6 {
            write_text(&mut out.text, out.spaces.clone(), format!("mov {}, rax", REGS[arg]).as_str());
            write_text(&mut out.text, out.spaces.clone(), format!("pop rax").as_str());
        } else {
            write_text(&mut out.text, out.spaces.clone(), format!("mov r15, rax").as_str());
            write_text(&mut out.text, out.spaces.clone(), format!("pop rax").as_str());
            write_text(&mut out.text, out.spaces.clone(), format!("push r15").as_str());
        }
    }
    write_text(&mut out.text, out.spaces.clone(), format!("call {}", statement.fn_ident).as_str());
}

pub fn compile(functab: HashMap<String, FuncTableVal>, globals: Vec<GlobalVar>) {
    let mut out = CompiledAsm { text: String::new(), data: String::new(), rodata: String::new(), string_literals: Vec::new(), num_strings: 0, spaces: String::new() };
    for (key, val) in functab.into_iter() {
        out.spaces.clear();
        write_text(&mut out.text, out.spaces.clone(), format!("\n{}: push rbp", key).as_str());
        for space in key.chars() {
            out.spaces.push_str(" ");
        }
        out.spaces.push_str("  ");
        let mut all_vars = Vec::new();
        for (i, arg) in val.signature.args.iter().enumerate() {
            assert!(i < 6, "Function calls with more than 6 args are not yet allowed.");
            write_text(&mut out.text, out.spaces.clone(), format!("push {}", REGS[i]).as_str());
            all_vars.push(arg.val.clone());
        }
        // init local vars
        for statement in &val.statements {
            if let Statement::Define(s) = statement {
                write_text(&mut out.text, out.spaces.clone(), format!("push 0 ;; {}", s.identifier).as_str());
                all_vars.push(s.identifier.clone());
            }
        }
        write_text(&mut out.text, out.spaces.clone(), "mov rbp, rsp");
        // now actually compile the statements
        let mut has_early_ret = false;
        for statement in val.statements.clone() {
            match statement {
                Statement::Assign(v) => { compile_assign(&mut out, v, all_vars.clone(), globals.clone()) },
                Statement::Define(v) => { compile_define(&mut out, v, all_vars.clone(), globals.clone()) },
                Statement::InlineAsm(v)=> { compile_inline_asm(&mut out, v, all_vars.clone(), globals.clone()) },
                Statement::FuncCall(v) => { compile_func_call(&mut out, v, all_vars.clone(), globals.clone()) },
                Statement::Return(v) => { compile_return(&mut out, v, all_vars.clone(), globals.clone(), val.clone()); has_early_ret = true; break },
                _ => { panic!("Cannot compile this statement") }
            }
        };
        if has_early_ret { continue }
        for (i, v) in all_vars.iter().enumerate() {
            write_text(&mut out.text, out.spaces.clone(), format!("pop rdi").as_str());
        }
        write_text(&mut out.text, out.spaces.clone(), "pop rbp");
        write_text(&mut out.text, out.spaces.clone(), "mov rax, 0");
        write_text(&mut out.text, out.spaces.clone(), "ret");
    }
    
    out.spaces.clear();

    for strlit in 0..out.num_strings {
        write_text(&mut out.rodata, out.spaces.clone(), format!("strlit{}: db {}", strlit, out.string_literals[strlit]).as_str());
    }

    let mut file = File::create("out.asm").expect("Couldn't open file");
    let _ = file.write_all(format!("[BITS 64]\n\nglobal _start\n").as_bytes());
    let _ = file.write_all(format!("\nsection .text\n{}\n", out.text).as_bytes());
    let _ = file.write_all(format!("section .data\n\n{}", out.data).as_bytes());
    let _ = file.write_all(format!("section .rodata\n\n{}", out.rodata).as_bytes());
}








