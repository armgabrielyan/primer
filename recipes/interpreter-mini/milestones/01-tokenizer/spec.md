# Milestone 01: Tokenizer

## Goal

Build a tokenizer that converts source text into a JSON token stream for a tiny expression language.

## What you'll build

- `mini_lang.py`: the interpreter entrypoint
- a `tokens` command that accepts source text and prints JSON

Support these token categories:

- integers
- identifiers
- `+`
- `-`
- `*`
- `/`
- `(`
- `)`
- `=`

Use this token shape:

```json
{
  "type": "NUMBER",
  "value": "42"
}
```

## Acceptance criteria

- `python3 mini_lang.py tokens "sum = 1 + 2"` exits with code `0`
- The command prints valid JSON
- The JSON is a list of token objects
- `sum = 1 + 2` produces these token types in order:
  - `IDENT`
  - `EQUAL`
  - `NUMBER`
  - `PLUS`
  - `NUMBER`
- Whitespace is ignored
- Unknown characters cause a non-zero exit and a helpful error message

## Files to create in project workspace

- `mini_lang.py`

## Suggested implementation notes

- Start with a simple left-to-right scanner
- Keep token type names explicit and stable
- Treat the tokenizer as its own stage, even if the program is still a single file

## Resources

- https://docs.python.org/3/library/json.html
- https://docs.python.org/3/library/re.html
