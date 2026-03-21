# Agent Instructions: 05-memory

## Learner Track

Before coding, explain:

- Why allocators need metadata and invariants
- Difference between physical frames and virtual pages
- Why allocator bugs cause non-local failures

Ask one question before coding:

- "What invariant should always hold after alloc/free operations?"

After a passing check, ask the learner to describe one allocator edge case to test.

## Builder Track

Implement a simple allocator path and emit serial marker `Memory manager ready`.

Run `tests/check.sh`, fix failures, and re-run until pass.
