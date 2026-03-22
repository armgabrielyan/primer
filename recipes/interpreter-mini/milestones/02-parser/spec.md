# Milestone 02: Parser

## Goal

Parse arithmetic expressions into an AST with correct precedence and parentheses handling.

## What you'll build

- a `parse` command that accepts an expression string and prints AST JSON
- support for binary expressions using:
  - `+`
  - `-`
  - `*`
  - `/`
  - parentheses

Use this AST shape:

```json
{
  "type": "BinaryExpr",
  "operator": "+",
  "left": { "type": "NumberLiteral", "value": 1 },
  "right": { "type": "NumberLiteral", "value": 2 }
}
```

## Acceptance criteria

- `python3 mini_lang.py parse "1 + 2 * 3"` prints valid JSON
- The AST respects precedence so the root operator is `+` and the right child operator is `*`
- `python3 mini_lang.py parse "(1 + 2) * 3"` prints valid JSON
- Parentheses change the AST so the root operator is `*` and the left child operator is `+`
- Invalid syntax exits non-zero and prints a helpful error message

## Files to update in project workspace

- `mini_lang.py`

## Suggested implementation notes

- A recursive descent parser is a good fit here
- Separate expression levels by precedence
- Keep the AST JSON stable because later checks and tests rely on it

## Resources

- https://craftinginterpreters.com/parsing-expressions.html
