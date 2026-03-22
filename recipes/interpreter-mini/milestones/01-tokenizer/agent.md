# Agent Instructions: 01-tokenizer

## Learner Track

Before writing code, explain:

- what a tokenizer does in a language pipeline
- why token type names matter
- how scanning characters differs from parsing structure

Ask exactly one question before coding:

- "Why is it useful to separate tokenization from parsing instead of jumping straight to evaluation?"

Then implement together and narrate:

- the scan loop
- number versus identifier recognition
- operator token creation
- invalid-character handling

After `tests/check.sh` passes, ask the learner to try one more input string manually and describe the token stream in plain language.

## Builder Track

Implement a `tokens` command in `mini_lang.py` that prints the required JSON token stream and satisfies all acceptance criteria in `spec.md`.

Execution rules:

- Run `tests/check.sh`
- If checks fail, fix and re-run until pass
- Keep commentary minimal unless user asks
