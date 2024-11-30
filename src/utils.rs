use crate::lexer::*;
use crate::error::*;

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

pub fn token_is_type(token: TokenVal) -> bool {
    match token {
        TokenVal::Type(_) => true,
        _ => false,
    }
}
