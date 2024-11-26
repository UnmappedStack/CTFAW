/* Parser for the language. Primarily creates a function table and uses the statement parser in
 * `statements.rs` to create a statement list for the program.
 */

#![allow(dead_code, unused_variables)]

use crate::error::*;
use std::collections::HashMap;
use crate::optimisation;
use crate::ast::*;
use crate::statements::*;
use crate::lexer::*;

#[derive(Debug, Clone)]
pub struct FuncArg {
    pub arg_type: Type,
    pub val: String,
}

#[derive(Debug, Clone)]
pub struct FuncSig {
    pub ret_type: Type,
    pub args: Vec<FuncArg>,
}

#[derive(Debug, Clone)]
pub struct FuncTableVal {
    pub signature: FuncSig,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct GlobalVar {
    pub identifier: String,
    pub typ: Type,
    pub val: u64,
}

pub fn get_ident(tok: &Token) -> String {
    let tok_val = tok.val.clone();
    if let TokenVal::Literal(v) = tok_val {
        if let LitVal::Ident(val) = v.val.clone() {
            val
        } else {
            report_err(Component::PARSER, tok.clone(), "Expected identifier, got something else. Failed to compile.");
            String::from("ctfaw_failure")
        }
    } else {
        report_err(Component::PARSER, tok.clone(), "Expected identifier, got something else. Failed to compile.");
        String::from("ctfaw_failure")
    }
}

pub fn get_str(tok: &Token) -> String {
    let tok_val = tok.val.clone();
    if let TokenVal::Literal(v) = tok_val {
        if let LitVal::Str(val) = v.val.clone() {
            val
        } else {
            report_err(Component::PARSER, tok.clone(), "Expected string literal, got something else.");
            String::from("ctfaw_failure")
        }
    } else {
        report_err(Component::PARSER, tok.clone(), "Expected string literal, got something else.");
        String::from("ctfaw_failure")
    }
}

/* Function declaration syntax:
 * func fnName(arg: type, arg: type) -> retType {}
 *  -- OR --
 * func fnName(arg: type, arg: type) {}
 *
 * Note that if a return type isn't specified, then U32 is assumed and 0 will be returned by
 * default. In my opinion this is cleaner than using a void type.
 */
pub fn parse(tokens_whole: Vec<Token>, global_vars: &mut Vec<GlobalVar>) -> HashMap<String, FuncTableVal> {
    let tokens: Vec<TokenVal> = tokens_whole.clone().into_iter()
        .map(|parent| parent.val)
        .collect();
    let mut function_table = HashMap::new();
    let mut skip = 0;
    for (i, token) in tokens.iter().enumerate() {
        if skip > 0 {
            skip -= 1;
            continue;
        }
        assert_report(*token != TokenVal::Let, Component::PARSER, tokens_whole[i].clone(), "Global variables must be constant, but one was defined with the `let` keyword.");
        if *token == TokenVal::Const {
            let mut n = 0;
            let mut global_iter = tokens.iter().skip(i);
            while let Some(this_token) = global_iter.next() {
                if *this_token == TokenVal::Endln { break }
                n += 1;
            }
            let global_def_statement = if let Statement::Define(v) = parse_define_statement(Vec::from(&tokens_whole[i..i + n + 1])) {Some(v)} else { unreachable!() };
            let (can_fold, new_ast) = optimisation::fold_expr(global_def_statement.as_ref().unwrap().expr.clone());
            assert_report(can_fold, Component::PARSER, tokens_whole[i + 1].clone(), "Global constants cannot contain identifiers, function calls, or anything besides numbers & operations.");
            let val = if let BranchChildVal::Int(v) = global_def_statement.clone().unwrap().expr.val { v } else { unreachable!() };
            global_vars.push(GlobalVar { identifier: global_def_statement.clone().unwrap().identifier, typ: global_def_statement.unwrap().def_type, val });
        }
        if *token != TokenVal::Func { continue }
        // it's a function declaration indeed. Get the identifier.
        let identifier = get_ident(&tokens_whole[i + 1]);
        // Get the args
        let mut args = Vec::new();
        let mut decl_iter = tokens.iter().skip(i + 2);
        let next = decl_iter.next().unwrap().clone();
        assert_report(next == TokenVal::Lparen, Component::PARSER, tokens_whole[i + 3].clone(), "Expected token after function identifier to be `(`, got something else instead.");
        let mut num_open_lparens = 1;
        let mut offset = i + 4;
        while let Some(this_token) = decl_iter.next() {
            offset += 1;
            if *this_token == TokenVal::Comma { continue }
            if *this_token == TokenVal::Lparen { num_open_lparens += 1; continue }
            if *this_token == TokenVal::Rparen {
                num_open_lparens -= 1;
                if num_open_lparens == 0 { break }
                continue;
            }
            let identifier = get_ident(&tokens_whole[i + 4]);
            offset += 2;
            assert_report(*decl_iter.next().unwrap() == TokenVal::Colon, Component::PARSER, tokens_whole[i + 5].clone(), "Expected `:` after identifier in arg list of function declaration, got something else.");
            let argtype = if let TokenVal::Type(v) = decl_iter.next().unwrap().clone() {
                v
            } else {
                report_err(Component::PARSER, tokens_whole[i + 6].clone(), "Expected type after colon (`:`) in function signature arg list, got something else instead.");
                unreachable!();
            };
            args.push(FuncArg {
                arg_type: argtype,
                val: identifier,
            });
        }
        let next_tok = decl_iter.next().unwrap();
        let mut to_check = next_tok.clone();
        let mut is_specified = false;
        let rettype = if *next_tok == TokenVal::Arrow {
            is_specified = true;
            offset += 2;
            let result = if let TokenVal::Type(mut t) = decl_iter.next().unwrap().clone() {
                to_check = decl_iter.next().unwrap().clone();
                for tok in &tokens_whole[i + 6..] {
                    if tok.val != TokenVal::Ops(Operation::Star) {break}
                    to_check = decl_iter.next().unwrap().clone();
                    t.ptr_depth += 1;
                }
                t
            } else {
                report_err(Component::PARSER, tokens_whole[i + 7].clone(), "Expected type after -> in function declaration specifying return type, got something else.");
                unreachable!();
            };
            result
        } else {
            Type {val: TypeVal::U32, ptr_depth: 0}
        };
        let o = if is_specified { 6 } else { 4 } as usize;
        assert_report(to_check == TokenVal::Lbrace, Component::PARSER, tokens_whole[i + o].clone(), "Expected left brace (`{`) after function declaration, got something else.");
        let mut num_open_lbraces = 1;
        let mut n = 0;
        while let Some(this_token) = decl_iter.next() {
            if *this_token == TokenVal::Lbrace { num_open_lbraces += 1; continue }
            if *this_token == TokenVal::Rbrace {
                num_open_lbraces -= 1;
                if num_open_lbraces == 0 { break }
                continue;
            }
            n += 1;
        }
        let statement_tokens = &tokens_whole[offset..offset + n];
        skip = n + 1;
        let statements_before_parse: Vec<_> = statement_tokens
            .split(|e| e.val == TokenVal::Endln)
            .filter(|v| !v.is_empty())
            .collect();
        let mut statements = Vec::new();
        for statement in statements_before_parse {
            let mut statement_vec = Vec::from(statement);
            statement_vec.push(Token {val: TokenVal::Endln, row: 0, col: 0 });
            statements.push(parse_statement(statement_vec));
        }
        function_table.insert(
            identifier.clone(),
            FuncTableVal {
                signature: FuncSig {
                    ret_type: rettype,
                    args,
                },
                statements,
            }
        );
    }
    function_table
}
