# Milestone 04: Variables and Tests

## Goal

Extend the language with assignment-based programs loaded from a file and protect the behavior with tests.

## What you'll build

- a `run` command that executes a source file
- variable assignment using `name = expression`
- identifier lookup inside later expressions
- a `tests/` directory with standard-library unit tests

Use this program model:

- the file contains one statement per line
- a statement is either an assignment or an expression
- the interpreter prints the value of the final expression statement

## Acceptance criteria

- Running this program prints `20`:

```txt
x = 2
y = x + 3
y * 4
```

- Using an unknown variable exits non-zero and prints a helpful error message
- `python3 -m unittest discover -s tests -p 'test_*.py'` exits with code `0`
- The tests cover at least one real language behavior path

## Files to create or update in project workspace

- `mini_lang.py`
- `tests/`

## Suggested implementation notes

- Keep program execution separate from single-expression evaluation
- Use an environment dictionary for variable bindings
- Reuse earlier tokenizer, parser, and evaluator stages instead of bypassing them

## Resources

- https://docs.python.org/3/library/unittest.html
