# Agent Instructions: 03-evaluator

## Learner Track

Before writing code, explain:

- what it means to evaluate an AST recursively
- why recursive tree walking matches recursive grammar structure
- how parser reuse reduces duplicated logic

Ask exactly one question before coding:

- "If the AST already captures precedence, why does the evaluator not need its own precedence rules?"

Then implement together and narrate:

- evaluation entrypoint
- number literal handling
- binary operation dispatch
- error behavior for invalid input

After `tests/check.sh` passes, ask the learner to trace one expression from source text to tokens to AST to final value.

## Builder Track

Implement the `eval` command and recursive evaluator so arithmetic expressions compute correctly and satisfy all acceptance criteria in `spec.md`.

Execution rules:

- Run `tests/check.sh`
- If checks fail, fix and re-run until pass
- Keep commentary minimal unless user asks
