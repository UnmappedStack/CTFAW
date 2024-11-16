#![allow(dead_code, unused_variables)]

use crate::statements::*;
use crate::lexer::*;
use crate::ast::*;
use std::fmt::Write;

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

pub fn compile_define(out: &mut CompiledAsm, statement: DefineStatement) {
    let _ = out.data.write_fmt(format_args!("{}: dq 0", statement.identifier));
    compile_expression(out, statement.expr);
    let _ = out.text.write_fmt(format_args!("mov [{}], rax", statement.identifier));
}

pub fn compile_assign(out: &mut CompiledAsm, statement: AssignStatement) {
    compile_expression(out, statement.expr);
    let _ = out.text.write_fmt(format_args!("mov [{}], rax", statement.identifier));
}

pub fn compile_inline_asm(statement: InlineAsmStatement) {
    let mut out = CompiledAsm{ text: String::new(), data: String::new() };
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
    println!("\nsection .text\n\n{}\n", out.text);
    println!("\nsection .data\n\n{}\n", out.data);
}






