# CTFAW
Stands for Compiler To Fuck Around With, and is pronounced "see-tee-foh".

This is just a toy compiler that I'm messing with in Rust. It's not meant to be good. It isn't even unlikely for it to randomly crash.

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
- [ ] Parser
    - [X] Basic expression parsing & AST generation
    - [X] Support bracketes in expression parsing
    - [ ] Basic statement lists generation
    - [ ] Function table generation
- [ ] Semantic analysis

**Backend**
The exact components are to be determined.

You may notice that this is missing middle end. I am currently skipping it, although I may come back to it.
