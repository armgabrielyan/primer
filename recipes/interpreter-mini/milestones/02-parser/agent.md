# Agent Instructions: 02-parser

## Learner Track

Before writing code, explain:

- why precedence exists in arithmetic expressions
- how parentheses override default precedence
- why AST structure is more useful than evaluating during parsing

Ask exactly one question before coding:

- "What tree shape should `1 + 2 * 3` produce if multiplication binds tighter than addition?"

Then implement together and narrate:

- parser entrypoint
- precedence layers
- primary expressions and parentheses
- syntax error reporting

After `tests/check.sh` passes, ask the learner to compare the AST for `1 + 2 * 3` with the AST for `(1 + 2) * 3` and explain the difference.

## Builder Track

Implement the `parse` command and AST generation so arithmetic expressions parse correctly and satisfy all acceptance criteria in `spec.md`.

Execution rules:

- Run `tests/check.sh`
- If checks fail, fix and re-run until pass
- Keep commentary minimal unless user asks
