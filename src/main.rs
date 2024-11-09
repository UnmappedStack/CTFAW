mod lexer;
mod parser;
mod ast;
mod error;

fn main() {
    let input: &str = "asm(\"mov rsp, rax\" : \"rax\" | var : \"rsp\" | var2 : \"rsp\", \"rax\");"; // just as a test
    println!("Full input: {}", input);

    let tokens = lexer::lex(input);
    parser::parse_statement(tokens);
}
