# CTFAW
Stands for Compiler To Fuck Around With, and is pronounced "see-tee-foh".

This is just a toy compiler that I'm messing with in Rust for a custom language, which compiles down to NASM assembly. It's not meant to be good. It isn't even unlikely for it to randomly crash.

It primarily uses the file extension `.ctf`, however you can really use whatever you like.

Don't use it. It's still very much a WIP.

## More information

- See syntax definitions in `SYNTAX.txt`, which is in the Backus-Naur format. This is the basis of the parser and how the AST will be constructed.
- See how components such as the parser and backend codegen work in the `design/` directory.

## Quickstart

```shell
$ cargo run <input file>
```
This will build an assembly file (stored in `./out.asm`), then call LD and NASM to build a final executable (`./out`). It is run automatically.

## Roadmap

- [X] Lexer (tokenisation)
- [X] Basic parser (AST generation, statement list generation, function table generation)
- [X] Basic backend and codegen
- [X] Support escape characters (`\n`, `\t`, `\"`, etc.)
- [X] String literal support
- [X] Return statement support
- [X] Constant folding (Optimisation)
- [X] Define & assign statements
- [X] Inline assembly
- [X] Function calls
- [X] Local functions & scope
- [X] Referencing
- [X] Dereferencing
- [ ] Type checking
- [ ] Externs & libc compatibility
- [ ] Logical operations (`&&`, `||`, `!`, etc.)
- [ ] Bitwise operations (`>>`/`<<`, `|`, `&`, `~`, etc.)
- [ ] Arrays
- [ ] Logical blocks
    - [ ] If
    - [ ] Else
    - [ ] Elseif
    - [ ] While
    - [ ] For
- [ ] Decent error handling
- [ ] Semantic analysis
- [ ] More to come...

## License & Contributing

This project is under the Mozilla Public License 2.0. See the details in the `LICENSE` file.

I'm currently not open to contributions, as this is:

1. This is my way of learning Rust (it's my second Rust project)
2. A personal project to learn the concepts behind compilers for fun.
