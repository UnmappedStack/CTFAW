/* The parser which constructs a function table, statement lists, and ASTs, which the back-end will step through
 * to build the final program. This makes the program much easier to comprehend for my bestie
 * boodle troodles cpu friend :)
 */

#![allow(unused_variables)]

use crate::lexer::*;

/* Parses an expression into an AST.
 * Takes a list of tokens, all of which must be an operator, grouping symbol, number, or
 * identifier. Returns an ASTNode which is the root of an AST for this expression. */
pub fn parse_expression(tokens: Vec<Token>) {
    println!("yo im parser");
}
