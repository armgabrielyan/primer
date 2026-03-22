# Agent Instructions: 02-complete

## Learner Track

Before writing code, explain:

- why stable ids matter once tasks can be updated
- the difference between data lookup by index and lookup by identifier
- why CLI tools should exit non-zero on invalid operations

Ask exactly one question before coding:

- "If a task is deleted later, why would using list position as the task identity become risky?"

Then implement together and narrate:

- id lookup
- state updates for `done`
- error handling for missing ids
- status-focused list output

After `tests/check.sh` passes, ask the learner to create two tasks and mark only one of them done so they can inspect the changed JSON by hand.

## Builder Track

Implement the `done` command, stable id handling, and visible task status output so the CLI satisfies all acceptance criteria in `spec.md`.

Execution rules:

- Run `tests/check.sh`
- If checks fail, fix and re-run until pass
- Keep commentary minimal unless user asks
