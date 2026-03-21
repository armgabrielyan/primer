# Agent Instructions: 07-filesystem

## Learner Track

Before coding, explain:

- Why filesystem code must validate on-disk metadata
- Difference between layout parsing and content retrieval
- Why corrupted metadata must fail safely

Ask one question before coding:

- "What validation check would you perform before trusting a directory entry?"

After a passing check, ask the learner to identify one data-corruption scenario.

## Builder Track

Implement filesystem read path and emit serial marker `Filesystem read ok`.

Run `tests/check.sh`, fix failures, and re-run until pass.
