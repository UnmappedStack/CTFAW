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
const DEBUG: bool = true;

#[derive(Clone)]
pub struct CompiledAsm {
    text: String,
    data: String,
    rodata: String,
    string_literals: Vec<String>,
    num_strings: usize,
    spaces: String,
}

#[derive(Clone)]
pub struct LocalVar {
    ident: String,
    typ: Type,
}

// Takes a type and outputs the size (in bytes)
fn type_to_size(typ: Type) -> u64 {
    if typ.ptr_depth > 0 { return 8 }
    match typ.val {
        TypeVal::U8 | TypeVal::Char | TypeVal::Boolean => 1,
        TypeVal::U16 => 2,
        TypeVal::U32 => 4,
        TypeVal::U64 | TypeVal::F64 => 8,
        _ => todo!()
    }
}

// Takes a (64 bit) register and a type, outputs the corresponding register name of the right size
fn register_of_size(original: &str, typ: Type) -> String {
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

fn write_text(txt: &mut String, spaces: String, new: &str) {
    if new.as_bytes()[0] == ';' as u8 && !DEBUG {return}
    let _ = txt.write_str(spaces.as_str());
    let replaced = new.replace("\n", format!("\n{spaces}").as_str());
    let _ = txt.write_str(replaced.as_str());
    let _ = txt.write_str("\n");
}

fn get_local_offset(v: String, allvars: Vec<LocalVar>) -> usize {
    match allvars.iter().position(|s| s.ident == v) {
        Some(val) => {
            val * 8
        },
        None => {
            panic!("Variable not defined in current scope.")
        }
    }
}

fn get_var_loc(v: String, locals: Vec<LocalVar>, globals: Vec<GlobalVar>) -> (String, Type) {
    let local_pos = locals.iter().position(|s| s.ident == v);
    match local_pos {
        Some(val) => {
            (format!("{} [rbp + {}]", ptr_ident_of_size(locals[val].typ.clone()), val * 8), locals[val].typ.clone())
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

/* Operands are in rax and rbx, and returns in rax. */
fn compile_operation(out: &mut CompiledAsm, op: Operation, rettype: Type) {
    let rax_sized = register_of_size("rax", rettype.clone());
    let rbx_sized = register_of_size("rbx", rettype);
    match op {
        Operation::Star => {
            write_text(&mut out.text, out.spaces.clone(), format!("mul {}", rbx_sized).as_str());
        },
        Operation::Add => {
            write_text(&mut out.text, out.spaces.clone(), format!("add {}, {}", rax_sized, rbx_sized).as_str());
        },
        Operation::Sub => {
            write_text(&mut out.text, out.spaces.clone(), format!("sub {}, {}", rbx_sized, rax_sized).as_str());
            write_text(&mut out.text, out.spaces.clone(), format!("mov {}, {}", rax_sized, rbx_sized).as_str());
        },
        Operation::Div => {
            write_text(&mut out.text, out.spaces.clone(), format!("div {}", rbx_sized).as_str());
        },
        _ => {
            panic!("Unsupported operation.")
        }
    }
}

/* The result of a single AST branch is stored in RAX. */
fn compile_ast_branch(out: &mut CompiledAsm, branch: BranchChild, allvars: Vec<LocalVar>, globals: Vec<GlobalVar>, rettype: Type) {
    let rax_sized = register_of_size("rax", rettype.clone());
    match branch.val {
        BranchChildVal::Branch(val) => {
            // compile it as a branch
            compile_ast_branch(out, *val.left_val, allvars.clone(), globals.clone(), rettype.clone());
            write_text(&mut out.text, out.spaces.clone(), "push rax");
            compile_ast_branch(out, *val.right_val, allvars, globals, rettype.clone());
            write_text(&mut out.text, out.spaces.clone(), "pop rbx");
            compile_operation(out, val.op, rettype);
        },
        BranchChildVal::Char(val) => {
            write_text(&mut out.text, out.spaces.clone(), format!("mov {}, {}", rax_sized, val).as_str());
        },
        BranchChildVal::Int(val) => {
            // just return the value
            write_text(&mut out.text, out.spaces.clone(), format!("mov {}, {}", rax_sized, val).as_str());
        },
        BranchChildVal::Deref(val) => {
            let loc = get_var_loc(val, allvars, globals).0;
            write_text(&mut out.text, out.spaces.clone(), format!("mov rax, [{}]\nmov {}, [{}]", loc, rax_sized, rax_sized).as_str());
        },
        BranchChildVal::Ref(val) => {
            let loc = get_var_loc(val, allvars, globals).0;
            write_text(&mut out.text, out.spaces.clone(), format!("lea {}, {}", rax_sized, loc).as_str());
        },
        BranchChildVal::Ident(val) => {
            let loc = get_var_loc(val, allvars, globals).0;
            write_text(&mut out.text, out.spaces.clone(), format!("mov {}, {}", rax_sized, loc).as_str());
        },
        BranchChildVal::Fn(val) => {
            compile_func_call(out, val, allvars, globals);
        },
        BranchChildVal::StrLit(val) => {
            let mut stringchars: Vec<String> = val.chars().map(|c| (c as u8).to_string()).collect();
            stringchars.push(String::from("0")); // make sure it has a null terminator
            out.string_literals.push(stringchars.join(", "));
            write_text(&mut out.text, out.spaces.clone(), format!("lea rax, [strlit{}]", out.num_strings).as_str());
            out.num_strings += 1;
        },
        _ => {
            panic!("Not implemented yet, expressions can't yet handle floating point values.")
        }
    }
}

pub fn compile_expression(out: &mut CompiledAsm, ast: BranchChild, allvars: Vec<LocalVar>, globals: Vec<GlobalVar>, rettype: Type) {
    write_text(&mut out.text, out.spaces.clone(), ";; Solve expression");
    compile_ast_branch(out, ast, allvars, globals, rettype);
}

pub fn compile_define(out: &mut CompiledAsm, statement: DefineStatement, allvars: Vec<LocalVar>, globals: Vec<GlobalVar>) {
    //write_text(&mut out.data, format!("{}: dq 0", statement.identifier).as_str());
    let loc = get_var_loc(statement.identifier.clone(), allvars.clone(), globals.clone());
    compile_expression(out, statement.expr, allvars.clone(), globals.clone(), loc.1);
    write_text(&mut out.text, out.spaces.clone(), format!(";; Assign value to var {} and define it", statement.identifier).as_str());
    write_text(&mut out.text, out.spaces.clone(), format!("mov {}, {}", loc.0, register_of_size("rax", statement.def_type)).as_str());

}

pub fn compile_assign(out: &mut CompiledAsm, statement: AssignStatement, allvars: Vec<LocalVar>, globals: Vec<GlobalVar>) {
    let loc = get_var_loc(statement.identifier.clone(), allvars.clone(), globals.clone());
    compile_expression(out, statement.expr, allvars.clone(), globals.clone(), loc.clone().1);
    if statement.deref {
        write_text(&mut out.text, out.spaces.clone(), format!(";; Assign value to var {}", statement.identifier).as_str());
        write_text(&mut out.text, out.spaces.clone(), format!("mov rbx, {}", loc.0).as_str());
        write_text(&mut out.text, out.spaces.clone(), format!("mov [rbx], {}", register_of_size("rax", loc.1)).as_str());
        return
    }
    write_text(&mut out.text, out.spaces.clone(), format!("mov {}, {}", loc.0, register_of_size("rax", loc.1)).as_str());
}

pub fn compile_return(out: &mut CompiledAsm, expr: BranchChild, allvars: Vec<LocalVar>, globals: Vec<GlobalVar>, func: FuncTableVal) {
    write_text(&mut out.text, out.spaces.clone(), ";; Early return from function");
    compile_expression(out, expr, allvars.clone(), globals.clone(), func.signature.ret_type); // this already puts it into rax
    write_text(&mut out.text, out.spaces.clone(), format!("add rsp, {}", allvars.len() * 8).as_str());
    write_text(&mut out.text, out.spaces.clone(), "pop rbp");
    write_text(&mut out.text, out.spaces.clone(), "ret");
}

pub fn compile_inline_asm(out: &mut CompiledAsm, statement: InlineAsmStatement, allvars: Vec<LocalVar>, globals: Vec<GlobalVar>) {
    write_text(&mut out.text, out.spaces.clone(), ";; Inline assembly");
    for clobber in &statement.clobbers {
        write_text(&mut out.text, out.spaces.clone(), format!("push {}", clobber).as_str());
    }
    for input in statement.inputs {
        write_text(&mut out.text, out.spaces.clone(), format!("mov {}, {}", input.register, get_var_loc(input.identifier, allvars.clone(), globals.clone()).0).as_str());
    }
    write_text(&mut out.text, out.spaces.clone(), format!("{}", statement.asm).as_str());
    for output in statement.outputs {
        write_text(&mut out.text, out.spaces.clone(), format!("mov {}, [{}]", get_var_loc(output.identifier, allvars.clone(), globals.clone()).0, output.register).as_str());
    }
    for clobber in &statement.clobbers {
        write_text(&mut out.text, out.spaces.clone(), format!("pop {}", clobber).as_str());
    }
}

pub fn compile_func_call(out: &mut CompiledAsm, statement: FuncCallStatement, allvars: Vec<LocalVar>, globals: Vec<GlobalVar>) {
    for arg in 0..statement.args.len() {
        write_text(&mut out.text, out.spaces.clone(), "push rax");
        compile_expression(out, statement.args[arg].clone(), allvars.clone(), globals.clone(), Type {val: TypeVal::U64, ptr_depth: 0});
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
            all_vars.push(LocalVar {
                ident: arg.val.clone(),
                typ: arg.arg_type.clone(),
            });
        }
        // init local vars
        for statement in &val.statements {
            if let Statement::Define(s) = statement {
                all_vars.push(LocalVar {
                    ident: s.identifier.clone(),
                    typ: s.def_type.clone(),
                });
            }
        }
        write_text(&mut out.text, out.spaces.clone(), format!("sub rsp, {}", all_vars.len() * 8).as_str());
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
        write_text(&mut out.text, out.spaces.clone(), "pop rbp");
        write_text(&mut out.text, out.spaces.clone(), "xor rax, rax");
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








