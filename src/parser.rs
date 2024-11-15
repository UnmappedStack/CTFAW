/* Parser for the language. Primarily creates a function table and uses the statement parser in
 * `statements.rs` to create a statement list for the program.
 */

#![allow(dead_code, unused_variables)]

use std::collections::HashMap;
use crate::statements::*;
use crate::lexer::*;

#[derive(Debug)]
struct FuncArg {
    arg_type: Token,
    val: String,
}

#[derive(Debug)]
struct FuncSig {
    ret_type: Token,
    identifier: String,
    args: Vec<FuncArg>,
}

#[derive(Debug)]
struct FuncTableVal {
    signature: FuncSig,
    statements: Vec<Statement>,
}

/* Function declaration syntax:
 * func fnName(arg: type, arg: type) -> retType {}
 *  -- OR --
 * func fnName(arg: type, arg: type) {}
 *
 * Note that if a return type isn't specified, then U32 is assumed and 0 will be returned by
 * default. In my opinion this is cleaner than using a void type.
 */
pub fn parse(tokens: Vec<Token>) {
    let mut function_table = HashMap::new();
    for (i, token) in tokens.iter().enumerate() {
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
        let mut offset = 4;
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
        let rettype = if *decl_iter.next().unwrap() == Token::Arrow {
            offset += 2;
            decl_iter.next().unwrap().clone()
        } else {
            Token::U32
        };
        assert!(*decl_iter.next().unwrap() == Token::Lbrace, "Expected right brace (`{{`) after function declaration, got something else.");
        let mut num_open_lbraces = 1;
        let mut i = 0;
        while let Some(this_token) = decl_iter.next() {
            if *this_token == Token::Lbrace { num_open_lbraces += 1; continue }
            if *this_token == Token::Rbrace {
                num_open_lbraces -= 1;
                if num_open_lbraces == 0 { break }
                continue;
            }
            i += 1;
        }
        let statement_tokens = &tokens[offset..offset + i];
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
    println!("Table produced: {:?}", function_table);
}
