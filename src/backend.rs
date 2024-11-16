use crate::lexer::*;
use crate::ast::*;

/* The result of a single AST branch is stored in RAX. */

/* Operands are in rax and rbx, and returns in rax. */
fn compile_operation(op: Operation) {
    match op {
        Operation::Star => {
            println!("mul rbx");
        },
        Operation::Add => {
            println!("add rax, rbx");
        },
        Operation::Sub => {
            println!("sub rbx, rax");
            println!("mov rax, rbx");
        },
        Operation::Div => {
            println!("div rbx");
        },
        _ => {
            assert!(false, "Unsupported operation.");
        }
    }
}

fn compile_ast_branch(branch: BranchChild) {
    match branch {
        BranchChild::Branch(val) => {
            // compile it as a branch
            compile_ast_branch(*val.left_val);
            println!("push rax");
            compile_ast_branch(*val.right_val);
            println!("pop rbx");
            compile_operation(val.op);
        },
        BranchChild::Int(val) => {
            // just return the value
            println!("mov rax, {} ;; -- Move immediate --", val);
        },
        _ => {
            assert!(false, "Not implemented yet, expressions can't yet handle identifiers, function calls, or floating point values.");
        }
    }
}

pub fn compile_expression(ast: BranchChild) {
    println!("Tree: {:?}", ast);
    if let BranchChild::Branch(_) = ast {
        println!("push rbx");
        compile_ast_branch(ast);
        println!("pop rbx");
    } else {
        compile_ast_branch(ast);
    }
}
