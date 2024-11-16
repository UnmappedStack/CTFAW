# CTFAW
Stands for Compiler To Fuck Around With, and is pronounced "see-tee-foh".

This is just a toy compiler that I'm messing with in Rust for a custom language, which compiles down to NASM assembly. It's not meant to be good. It isn't even unlikely for it to randomly crash.

Don't use it. It's still very much a WIP.

## More information

- See syntax definitions in `SYNTAX.txt`, which is in the Backus-Naur format. This is the basis of the parser and how the AST will be constructed.
- See how components such as the parser and backend codegen work in the `design/` directory.

## Quickstart

Unfortunately CTFAW isn't yet at a point where it's ready to compile any file you want. To test out what's going on so far, simply run:
```shell
$ cargo run
```
This will output some debug information such as a list of tokens and ASTs, statement lists, and function tables.

## Roadmap

**Frontend**
- [X] Lexer (tokenisation)
- [X] Basic parser (more will still be added, this isn't all)
    - [X] Basic expression parsing & AST generation
    - [X] Support bracketes in expression parsing
    - [X] Basic statement lists generation
    - [X] Function table generation
- [ ] Semantic analysis

**Backend**
 - [X] Expression compilation (AST -> Assembly)
 - [X] Define & assign statements compilation
 - [X] Inline assembly compilation
 - [X] Function call compilation
 - [X] Compiling specific functions & their statement lists
 - [X] Local variables and scope
 - [ ] More to come...

You may notice that this is missing middle end. I am currently skipping it, although I may come back to it.

## License & Contributing

This project is under the Mozilla Public License 2.0. See the details in the `LICENSE` file.

I'm currently not open to contributions, as this is:

1. This is my way of learning Rust (it's my second Rust project)
2. A personal project to learn the concepts behind compilers for fun.
