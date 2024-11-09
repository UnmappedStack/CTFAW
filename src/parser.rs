/* Parses the list of tokens to create a function table and statement list. Also makes use of the
 * AST generator in `ast.rs`
 */

#![allow(dead_code, unused_variables)]

use crate::lexer::*;
use crate::ast::*;

// Some structures first need to be defined

struct DefineStatement {
    is_const: bool,
    identifier: String,
    def_type: Token,
    expr: BranchChild,
}

struct AssignStatement {
    identifier: String,
    expr: BranchChild,
}

struct FuncCallStatement {
    fn_ident: String,
    args: Vec<BranchChild>,
}

struct AsmIOEntry {
    register: String,
    identifier: String,
}

struct InlineAsmStatement {
    asm: String,
    inputs: Vec<AsmIOEntry>,
    outputs: Vec<AsmIOEntry>,
    clobbers: Vec<String>,
}

enum Statement {
    Define(DefineStatement),
    Assign(AssignStatement),
    FuncCall(FuncCallStatement),
    InlineAsm(InlineAsmStatement),
}

pub fn parse_statement(tokens: Vec<Token>) {
    
}
