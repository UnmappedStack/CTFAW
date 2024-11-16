mod lexer;
mod parser;
mod statements;
mod ast;
mod backend;
mod error;

fn main() {
    let input: &str = "asm(\";; Inline asm:\nmov rax, 15\nmov rbx, rax\n;; End inline asm\" : \"rax\" | var : \"rbx\" | var2 : \"rax\", \"rbx\");"; // just as a test
    println!("Full input: {}", input);

    let tokens = lexer::lex(input);
    let statement = if let statements::Statement::InlineAsm(val) = statements::parse_inline_asm_statement(tokens) {
        Some(val)
    } else {
        assert!(false, "Expected inline assembly statement.");
        None
    };
    backend::compile_inline_asm(statement.unwrap());
}
