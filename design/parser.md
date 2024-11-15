# Parser design
Vec<Token> is passed in, and it creates a function table, statement lists, and ASTs. Parser signature:

```rs
pub fn parse(tokens: Vec<Token>) -> HashMap<FuncSig, StatementList>
```

## Implementation
Go through each function declaration and add it to the function table. For each statement in the function:
1. Check the statement type (this can usually be told based on the first 1 or 2 tokens of the statement)
2. Add it to the statement list.
    - If there are any sub-statements within, parse them and point to it from this statement list entry recursively.
    - If there are any expressions within, parse them and create an AST which is pointed to by this statement list entry.

**Function table layout**
| **FnName** | **Signature** | **Statement list** |
|.----------.|.-------------.|.------------------.|
| ...        | ...           | ...                |
