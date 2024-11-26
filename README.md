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
This will by default build an executable, statically linked program with the filename `./out`.

You can also use the following options:

**-r** will automatically run the program after compilation is done. This cannot be used if `-S` or `-c` are used.

**-c** will output only an object file, rather than the final executable, which can be linked with other object files manually.

**-S** will output only the assembly file that it generates rather than the final executable.

**-o filename** will tell the compiler to name the output file `filename`. 

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
- [X] Decent error handling
- [X] Type checking
- [X] Pointer types
- [ ] Use specified data sizes in generated assembly
- [ ] Casting
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
- [ ] More to come...

## License & Contributing

This project is under the Mozilla Public License 2.0. See the details in the `LICENSE` file.

I'm currently not open to contributions, as this is:

1. This is my way of learning Rust (it's my second Rust project)
2. A personal project to learn the concepts behind compilers for fun.
