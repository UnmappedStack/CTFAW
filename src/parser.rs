/* Parser for the language. Primarily creates a function table and uses the statement parser in
 * `statements.rs` to create a statement list for the program.
 */

#![allow(dead_code, unused_variables)]

use std::collections::HashMap;
use crate::optimisation;
use crate::ast::*;
use crate::statements::*;
use crate::lexer::*;

#[derive(Debug, Clone)]
pub struct FuncArg {
    pub arg_type: Token,
    pub val: String,
}

#[derive(Debug, Clone)]
pub struct FuncSig {
    pub ret_type: Token,
    pub identifier: String,
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
    pub val: u64,
}

/* Function declaration syntax:
 * func fnName(arg: type, arg: type) -> retType {}
 *  -- OR --
 * func fnName(arg: type, arg: type) {}
 *
 * Note that if a return type isn't specified, then U32 is assumed and 0 will be returned by
 * default. In my opinion this is cleaner than using a void type.
 */
pub fn parse(tokens: Vec<Token>, global_vars: &mut Vec<GlobalVar>) -> HashMap<String, FuncTableVal> {
    let mut function_table = HashMap::new();
    let mut skip = 0;
    for (i, token) in tokens.iter().enumerate() {
        if skip > 0 {
            skip -= 1;
            continue;
        }
        if *token == Token::Let {
            assert!(false, "Global variables must be constant, but one was defined with the `let` keyword.");
        }
        if *token == Token::Const {
            let mut n = 0;
            let mut global_iter = tokens.iter().skip(i);
            while let Some(this_token) = global_iter.next() {
                if *this_token == Token::Endln { break }
                n += 1;
            }
            let global_def_statement = if let Statement::Define(v) = parse_define_statement(Vec::from(&tokens[i..i + n + 1])) {Some(v)} else { assert!(false, "unreachable"); None };
            let (can_fold, new_ast) = optimisation::fold_expr(global_def_statement.as_ref().unwrap().expr.clone());
            assert!(can_fold, "Global constants cannot contain identifiers, function calls, or anything besides numbers & operations.");
            let val = if let BranchChild::Int(v) = global_def_statement.clone().unwrap().expr { v } else { assert!(false, "unreachable"); 0 };
            global_vars.push(GlobalVar { identifier: global_def_statement.unwrap().identifier, val });
        }
        if *token != Token::Func { continue }
        // it's a function declaration indeed. Get the identifier.
        let identifier = if let Token::Ident(val) = &tokens[i + 1] {
            val.clone()
        } else {
            assert!(false, "Expected function identifier after `func` declaration, got something else. Failed to compile.");
            String::from("ctfaw_failure")
        };
        // Get the args
        let mut args = Vec::new();
        let mut decl_iter = tokens.iter().skip(i + 2);
        let next = decl_iter.next().unwrap().clone();
        assert!(next == Token::Lparen, "Expected token after function identifier to be `(`, got something else instead.");
        let mut num_open_lparens = 1;
        let mut offset = i + 4;
        while let Some(this_token) = decl_iter.next() {
            offset += 1;
            if *this_token == Token::Comma { continue }
            if *this_token == Token::Lparen { num_open_lparens += 1; continue }
            if *this_token == Token::Rparen {
                num_open_lparens -= 1;
                if num_open_lparens == 0 { break }
                continue;
            }
            let identifier = if let Token::Ident(val) = this_token {
                val.clone()
            } else {
                assert!(false, "Expected identifier in arg list of function declaration, got something else.");
                String::from("ctfaw_failure")
            };
            offset += 2;
            assert!(*decl_iter.next().unwrap() == Token::Colon, "Expected `:` after identifier in arg list of function declaration, got something else.");
            let argtype = decl_iter.next().unwrap().clone();
            args.push(FuncArg {
                arg_type: argtype,
                val: identifier,
            });
        }
        let next_tok = decl_iter.next().unwrap();
        let mut to_check = next_tok.clone();
        let rettype = if *next_tok == Token::Arrow {
            offset += 2;
            let result = decl_iter.next().unwrap().clone();
            to_check = decl_iter.next().unwrap().clone();
            result
        } else {
            Token::U32
        };
        assert!(to_check == Token::Lbrace, "Expected left brace (`{{`) after function declaration, got something else.");
        let mut num_open_lbraces = 1;
        let mut n = 0;
        while let Some(this_token) = decl_iter.next() {
            if *this_token == Token::Lbrace { num_open_lbraces += 1; continue }
            if *this_token == Token::Rbrace {
                num_open_lbraces -= 1;
                if num_open_lbraces == 0 { break }
                continue;
            }
            n += 1;
        }
        let statement_tokens = &tokens[offset..offset + n];
        skip = n + 1;
        let statements_before_parse: Vec<_> = statement_tokens
            .split(|e| *e == Token::Endln)
            .filter(|v| !v.is_empty())
            .collect();
        let mut statements = Vec::new();
        for statement in statements_before_parse {
            let mut statement_vec = Vec::from(statement);
            statement_vec.push(Token::Endln);
            statements.push(parse_statement(statement_vec));
        }
        function_table.insert(
            identifier.clone(),
            FuncTableVal {
                signature: FuncSig {
                    ret_type: rettype,
                    identifier,
                    args,
                },
                statements,
            }
        );
    }
    function_table
}
