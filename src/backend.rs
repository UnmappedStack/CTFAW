#![allow(dead_code, unused_variables)]

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
}

fn write_text(txt: &mut String, new: &str) {
    let _ = txt.write_str(new);
    let _ = txt.write_str("\n");
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
fn compile_ast_branch(out: &mut CompiledAsm, branch: BranchChild) {
    match branch {
        BranchChild::Branch(val) => {
            // compile it as a branch
            compile_ast_branch(out, *val.left_val);
            write_text(&mut out.text, "push rax");
            compile_ast_branch(out, *val.right_val);
            write_text(&mut out.text, "pop rbx");
            compile_operation(out, val.op);
        },
        BranchChild::Int(val) => {
            // just return the value
            let _ = out.text.write_fmt(format_args!("mov rax, {} ;; -- Move immediate --\n", val));
        },
        BranchChild::Fn(val) => {
            compile_func_call(out, val);
        },
        _ => {
            assert!(false, "Not implemented yet, expressions can't yet handle identifiers, function calls, or floating point values.");
        }
    }
}

pub fn compile_expression(out: &mut CompiledAsm, ast: BranchChild) {
    if let BranchChild::Branch(_) = ast {
        write_text(&mut out.text, "push rbx");
        compile_ast_branch(out, ast);
        write_text(&mut out.text, "pop rbx");
    } else {
        compile_ast_branch(out, ast);
    }
}

pub fn compile_define(out: &mut CompiledAsm, statement: DefineStatement, allvars: Vec<String>) {
    //let _ = out.data.write_fmt(format_args!("{}: dq 0", statement.identifier));
    compile_expression(out, statement.expr);
     match allvars.iter().position(|s| *s == statement.identifier) {
        Some(val) => {
            let _ = out.text.write_fmt(format_args!("mov [rsp + {}], rax", val * 8));
        },
        None => {
            assert!(false, "Can't assign to undefined variable.");
        }
    };

}

pub fn compile_assign(out: &mut CompiledAsm, statement: AssignStatement, allvars: Vec<String>) {
    compile_expression(out, statement.expr);
    match allvars.iter().position(|s| *s == statement.identifier) {
        Some(val) => {
            let _ = out.text.write_fmt(format_args!("mov [rsp + {}], rax", val * 8));
        },
        None => {
            assert!(false, "Can't assign to undefined variable.");
        }
    };
}

pub fn compile_inline_asm(out: &mut CompiledAsm, statement: InlineAsmStatement) {
    for clobber in &statement.clobbers {
        let _ = out.text.write_fmt(format_args!("push {}\n", clobber));
    }
    for input in statement.inputs {
        let _ = out.text.write_fmt(format_args!("mov {}, [{}]\n", input.register, input.identifier));
    }
    let _ = out.text.write_fmt(format_args!("{}\n", statement.asm));
    for output in statement.outputs {
        let _ = out.text.write_fmt(format_args!("mov [{}], {}\n", output.identifier, output.register));
    }
    for clobber in &statement.clobbers {
        let _ = out.text.write_fmt(format_args!("pop {}\n", clobber));
    }
}

pub fn compile_func_call(out: &mut CompiledAsm, statement: FuncCallStatement) {
    for arg in 0..statement.args.len() {
        write_text(&mut out.text, "push rax");
        compile_expression(out, statement.args[arg].clone());
        if arg < 6 {
            let _ = out.text.write_fmt(format_args!("mov {}, rax\npop rax\n", REGS[arg]));
        } else {
            let _ = out.text.write_fmt(format_args!("mov r15, rax\npop rax\npush r15\n"));
        }
    }
    let _ = out.text.write_fmt(format_args!("call {}\n", statement.fn_ident));
}

pub fn compile(functab: HashMap<String, FuncTableVal>) {
    let mut out = CompiledAsm { text: String::new(), data: String::new() };
    for (key, val) in functab.into_iter() {
        let _ = out.text.write_fmt(format_args!("{}:\n", key));
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
                Statement::Assign(v) => { compile_assign(&mut out, v, all_vars.clone()) },
                Statement::Define(v) => { compile_define(&mut out, v, all_vars.clone()) },
                Statement::InlineAsm(v)=> { compile_inline_asm(&mut out, v) },
                Statement::FuncCall(v) => { compile_func_call(&mut out, v) },
                _ => { assert!(false, "Cannot compile this statement") }
            }
        };
    }
    println!("\n[BITS 64]\nglobal _start");
    println!("\nsection .text\n\n{}\n", out.text);
    println!("section .data\n\n{}\n", out.data);
}








