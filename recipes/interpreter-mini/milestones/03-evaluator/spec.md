# Milestone 03: Evaluator

## Goal

Evaluate arithmetic expressions from the AST and print the numeric result.

## What you'll build

- an `eval` command that accepts an expression string and prints the computed result
- evaluator support for:
  - integer literals
  - `+`
  - `-`
  - `*`
  - `/`
  - parentheses

## Acceptance criteria

- `python3 mini_lang.py eval "1 + 2 * 3"` prints `7`
- `python3 mini_lang.py eval "(1 + 2) * 3"` prints `9`
- `python3 mini_lang.py eval "8 / 2 + 1"` prints `5`
- Invalid syntax exits non-zero and prints a helpful error message

## Files to update in project workspace

- `mini_lang.py`

## Suggested implementation notes

- Reuse the parser instead of re-interpreting raw strings directly
- Keep evaluation logic separate from parsing logic
- Decide whether division should produce integer or floating-point results, then keep it consistent

## Resources

- https://docs.python.org/3/library/operator.html
