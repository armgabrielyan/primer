# Agent Instructions: 04-filters-tests

## Learner Track

Before writing code, explain:

- why filtering is a natural extension of list behavior
- what a summary command adds for the user
- why tests are part of the product, not just cleanup work

Ask exactly one question before coding:

- "Would you rather test the CLI through subprocess calls, or extract logic into functions and test those directly?"

Then implement together and narrate:

- filter parsing for `list`
- count calculation for `summary`
- at least one meaningful test path
- how to run the test suite repeatedly while refining behavior

After `tests/check.sh` passes, ask the learner to run the tests once more and explain which user behavior each test is protecting.

## Builder Track

Implement status filters, a summary command, and a passing standard-library test suite that satisfies all acceptance criteria in `spec.md`.

Execution rules:

- Run `tests/check.sh`
- If checks fail, fix and re-run until pass
- Keep commentary minimal unless user asks
