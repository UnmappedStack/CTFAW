# Parser design
Vec<Token> is passed in, and it creates an AST (Abstract Syntax Tree). Parser signature:

```rs
pub fn parse(tokens: Vec<Token>) -> ExprNode
```

## Operation
1. From tokens, find the lowest priority token (priorities defined below)
2. Select which type of statement this is based on the token type. The way to do this is explained later.
3. Construct a branch of the right type. Repeat back to 1 for expressions etc., removing irrelevant symbols and keeping the ones needed for this next child branch.
4. Return the first node created which was the very lowest priority (which is the root of the AST)

### Symbols & Priorities
The type of statement can be told based on the token type, which was identified as the lowest priority token. The priorities are also defined here.

| **Statement type** |  **Symbol**      | **Priority**                    |
|:------------------:|:----------------:|:-------------------------------:|
| Expression         | N/A              | N/A                             |
| Literal            | N/A              | N/A                             |
| Grouping           | `(`/`)`          | 2                               |
| Unary              | `-`/`!`          | 3                               |
| Binary Operation   | Any Operator node| 4-6 (varies based on operation) |
| Operator           | N/A              | N/A                             |
| Define             | `=`              | 1                               |
| Assign             | `let`/`const`    | 0                               |

Statement types with the symbol set as `N/A` are not directly found, but are instead referred to by one of the statement types which *are* directly found, if that makes sense.

The binary operation priorities are as follows:

- **`+`/`-`**: 6
- **`*`/`/`**: 5
- **`^`**: 4

(Note that it follows classic order of operations, but inversed.)
