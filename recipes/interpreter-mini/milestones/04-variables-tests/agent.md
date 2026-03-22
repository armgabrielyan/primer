# Agent Instructions: 04-variables-tests

## Learner Track

Before writing code, explain:

- how variables introduce program state
- why reading a file of statements is different from evaluating a single expression
- why tests matter more once multiple interpreter phases depend on each other

Ask exactly one question before coding:

- "If `y = x + 3` appears before `x` is assigned, what should the interpreter do and why?"

Then implement together and narrate:

- statement parsing strategy
- environment storage
- unknown-variable errors
- at least one meaningful test path

After `tests/check.sh` passes, ask the learner to describe which earlier milestone outputs are now being reused by the final `run` command.

## Builder Track

Implement file-based program execution, variables, and a passing standard-library test suite that satisfies all acceptance criteria in `spec.md`.

Execution rules:

- Run `tests/check.sh`
- If checks fail, fix and re-run until pass
- Keep commentary minimal unless user asks
