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

pub struct CompiledAsm {
    text: String,
    data: String,
    rodata: String,
    num_strings: u64,
}

fn write_text(txt: &mut String, new: &str) {
    let _ = txt.write_str(new);
    let _ = txt.write_str("\n");
}

fn get_local_offset(v: String, allvars: Vec<String>) -> usize {
    match allvars.iter().position(|s| *s == v) {
        Some(val) => {
            val * 8
        },
        None => {
            assert!(false, "Variable not defined in current scope.");
            0
        }
    }
}

fn get_var_loc(v: String, locals: Vec<String>, globals: Vec<GlobalVar>, is_rdi: bool) -> String {
    let local_pos = locals.iter().position(|s| *s == v);
    match local_pos {
        Some(val) => format!("{} + {}", if is_rdi { "rdi" } else { "rsp" }, val * 8),
        None => {
            let global_pos = globals.iter().position(|s| s.identifier == v);
            match global_pos {
                Some(val) => v,
                None => { assert!(false, "Variable not defined in current scope."); String::from("0") }
            }
        }
    }
}

/* Operands are in rax and rbx, and returns in rax. */
fn compile_operation(out: &mut CompiledAsm, op: Operation) {
    match op {
        Operation::Star => {
            write_text(&mut out.text, "mul rbx");
        },
        Operation::Add => {
            write_text(&mut out.text, "add rax, rbx");
        },
        Operation::Sub => {
            write_text(&mut out.text, "sub rbx, rax");
            write_text(&mut out.text, "mov rax, rbx");
        },
        Operation::Div => {
            write_text(&mut out.text, "div rbx");
        },
        _ => {
            assert!(false, "Unsupported operation.");
        }
    }
}

/* The result of a single AST branch is stored in RAX. */
fn compile_ast_branch(out: &mut CompiledAsm, branch: BranchChild, allvars: Vec<String>, globals: Vec<GlobalVar>) {
    match branch {
        BranchChild::Branch(val) => {
            // compile it as a branch
            compile_ast_branch(out, *val.left_val, allvars.clone(), globals.clone());
            write_text(&mut out.text, "push rax");
            compile_ast_branch(out, *val.right_val, allvars, globals);
            write_text(&mut out.text, "pop rbx");
            compile_operation(out, val.op);
        },
        BranchChild::Int(val) => {
            // just return the value
            let _ = out.text.write_fmt(format_args!("mov rax, {}\n", val));
        },
        BranchChild::Ident(val) => {
            let _ = out.text.write_fmt(format_args!("mov rax, [{}]\n", get_var_loc(val, allvars, globals, true)));
        },
        BranchChild::Fn(val) => {
            compile_func_call(out, val, allvars, globals);
        },
        BranchChild::StrLit(val) => {
            let mut stringchars: Vec<String> = val.chars().map(|c| (c as u8).to_string()).collect();
            stringchars.push(String::from("0")); // make sure it has a null terminator
            let _ = out.rodata.write_fmt(format_args!("strlit{}: db {}\n", out.num_strings, stringchars.join(", ")));
            let _ = out.text.write_fmt(format_args!("mov rax, strlit{}\n", out.num_strings));
            out.num_strings += 1;
        },
        _ => {
            assert!(false, "Not implemented yet, expressions can't yet handle floating point values.");
        }
    }
}

pub fn compile_expression(out: &mut CompiledAsm, ast: BranchChild, allvars: Vec<String>, globals: Vec<GlobalVar>) {
    write_text(&mut out.text, "mov rdi, rsp");
    if let BranchChild::Branch(_) = ast {
        write_text(&mut out.text, "push rbx");
        compile_ast_branch(out, ast, allvars, globals);
        write_text(&mut out.text, "pop rbx");
    } else {
        compile_ast_branch(out, ast, allvars, globals);
    }
}

pub fn compile_define(out: &mut CompiledAsm, statement: DefineStatement, allvars: Vec<String>, globals: Vec<GlobalVar>) {
    //let _ = out.data.write_fmt(format_args!("{}: dq 0", statement.identifier));
    compile_expression(out, statement.expr, allvars.clone(), globals.clone());
    let _ = out.text.write_fmt(format_args!("mov [{}], rax\n", get_var_loc(statement.identifier, allvars, globals, false)));

}

pub fn compile_assign(out: &mut CompiledAsm, statement: AssignStatement, allvars: Vec<String>, globals: Vec<GlobalVar>) {
    compile_expression(out, statement.expr, allvars.clone(), globals.clone());
    let _ = out.text.write_fmt(format_args!("mov [{}], rax\n", get_var_loc(statement.identifier, allvars, globals, false)));
}

pub fn compile_return(out: &mut CompiledAsm, expr: BranchChild, allvars: Vec<String>, globals: Vec<GlobalVar>, func: FuncTableVal) {
    compile_expression(out, expr, allvars.clone(), globals.clone()); // this already puts it into rax
    for (i, v) in allvars.iter().enumerate() {
        let _ = out.text.write_fmt(format_args!("pop rdi\n"));
    }
    write_text(&mut out.text, "ret\n");
}

pub fn compile_inline_asm(out: &mut CompiledAsm, statement: InlineAsmStatement, allvars: Vec<String>, globals: Vec<GlobalVar>) {
    for clobber in &statement.clobbers {
        let _ = out.text.write_fmt(format_args!("push {}\n", clobber));
    }
    for input in statement.inputs {
        let _ = out.text.write_fmt(format_args!("mov {}, [{} - {}]\n", input.register,  get_var_loc(input.identifier, allvars.clone(), globals.clone(), false), 8 * statement.clobbers.len()));
    }
    let _ = out.text.write_fmt(format_args!("{}\n", statement.asm));
    for output in statement.outputs {
        let _ = out.text.write_fmt(format_args!("mov [{} - {}], {}\n", get_var_loc(output.identifier, allvars.clone(), globals.clone(), false), 8 * statement.clobbers.len(), output.register));
    }
    for clobber in &statement.clobbers {
        let _ = out.text.write_fmt(format_args!("pop {}\n", clobber));
    }
}

pub fn compile_func_call(out: &mut CompiledAsm, statement: FuncCallStatement, allvars: Vec<String>, globals: Vec<GlobalVar>) {
    for arg in 0..statement.args.len() {
        write_text(&mut out.text, "push rax");
        compile_expression(out, statement.args[arg].clone(), allvars.clone(), globals.clone());
        if arg < 6 {
            let _ = out.text.write_fmt(format_args!("mov {}, rax\npop rax\n", REGS[arg]));
        } else {
            let _ = out.text.write_fmt(format_args!("mov r15, rax\npop rax\npush r15\n"));
        }
    }
    let _ = out.text.write_fmt(format_args!("call {}\n", statement.fn_ident));
}

pub fn compile(functab: HashMap<String, FuncTableVal>, globals: Vec<GlobalVar>) {
    let mut out = CompiledAsm { text: String::new(), data: String::new(), rodata: String::new(), num_strings: 0 };
    for (key, val) in functab.into_iter() {
        let _ = out.text.write_fmt(format_args!("\n{}:\n", key));
        let mut all_vars = Vec::new();
        for (i, arg) in val.signature.args.iter().enumerate() {
            assert!(i < 6, "Function calls with more than 6 args are not yet allowed.");
            let _ = out.text.write_fmt(format_args!("push {}\n", REGS[i]));
            all_vars.push(arg.val.clone());
        }
        // init local vars
        for statement in &val.statements {
            if let Statement::Define(s) = statement {
                let _ = out.text.write_fmt(format_args!("push 0 ;; {}\n", s.identifier));
                all_vars.push(s.identifier.clone());
            }
        }
        // now actually compile the statements
        for statement in val.statements.clone() {
            match statement {
                Statement::Assign(v) => { compile_assign(&mut out, v, all_vars.clone(), globals.clone()) },
                Statement::Define(v) => { compile_define(&mut out, v, all_vars.clone(), globals.clone()) },
                Statement::InlineAsm(v)=> { compile_inline_asm(&mut out, v, all_vars.clone(), globals.clone()) },
                Statement::FuncCall(v) => { compile_func_call(&mut out, v, all_vars.clone(), globals.clone()) },
                Statement::Return(v) => { compile_return(&mut out, v, all_vars.clone(), globals.clone(), val.clone()) },
                _ => { assert!(false, "Cannot compile this statement") }
            }
        };
        for (i, v) in all_vars.iter().enumerate() {
            let _ = out.text.write_fmt(format_args!("pop rdi\n"));
        }
        write_text(&mut out.text, "mov rax, 0\nret\n");
    }
    
    for global in globals {
        let _ = out.rodata.write_fmt(format_args!("{}: dq {}\n", global.identifier, global.val));
    }

    let mut file = File::create("out.asm").expect("Couldn't open file");
    let _ = file.write_all(format!("[BITS 64]\nglobal _start").as_bytes());
    let _ = file.write_all(format!("\nsection .text\n{}\n", out.text).as_bytes());
    let _ = file.write_all(format!("section .data\n\n{}\n", out.data).as_bytes());
    let _ = file.write_all(format!("section .rodata\n\n{}\n", out.rodata).as_bytes());
}








