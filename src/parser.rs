/* The parser which constructs an Abstract Syntax Tree (AST), which the back-end will step through
 * to build the final program. This makes the program much easier to comprehend for my bestie
 * boodle troodles cpu friend :)
 */

use crate::lexer::*;

#[allow(dead_code)]
enum ExprNode {
    Unary(Box<UnaryNode>),
    BinOp(Box<BinOpNode>),
    Group(Box<ExprNode>),
    Identifier(String),
}

#[allow(dead_code)]
enum LitNode {
    Str(String),
    Int(u64),
    Float(f64),
    Bool(u8),
}

#[allow(dead_code)]
enum UnarySymbol {
    Negate,
    Not,
}

#[allow(dead_code)]
struct UnaryNode {
    sym: UnarySymbol,
    expr: Box<ExprNode>,
}

#[allow(dead_code)]
struct BinOpNode {
    expr1: Box<ExprNode>,
    oper: OperNode,
    expr2: Box<ExprNode>,
}

#[allow(dead_code)]
enum OperNode {
    Equ,
    NotEqu,
    LessThan,
    MoreThan,
    LessEqu,
    MoreEqu,
    Plus,
    Minus,
    Multiply,
    Divide,
}

#[allow(dead_code)]
enum Type {
    // Unsigned types
    U64,
    U32,
    U16,
    U8,
    // Signed integer types
    I64,
    I32,
    I16,
    I8,
    // Floating point value (always 64 bit signed)
    F64,
    // Other stuff
    Bool,
}

#[allow(dead_code)]
struct AssignNode {
    is_const: bool,
    identifier: String,
    var_type: Type,
    expr: Box<ExprNode>,
}

#[allow(dead_code)]
struct DefineNode {
    identifier: String,
    var_type: Type,
    expr: Box<ExprNode>,
}

#[allow(dead_code)]
enum Node {
    Expression(ExprNode),
    Literal(LitNode),
    Unary(UnaryNode),
    BinOp(BinOpNode),
    Operator(OperNode),
    Assign(AssignNode),
    Define(DefineNode),
}

pub fn parse(_tokens: Vec<Token>) {
    println!("Hi, I'm a parser.");
}
