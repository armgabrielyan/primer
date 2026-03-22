# Agent Instructions: 01-bootstrap

## Learner Track

Before writing code, explain:

- what a command-line entrypoint is
- why `argparse` is a good fit for a beginner CLI
- why storing tasks as JSON is a useful first persistence format

Ask exactly one question before coding:

- "What information do you think each task should store besides its title?"

Then implement together and narrate each step:

- command structure
- loading and saving tasks
- adding a task
- listing tasks when the file is empty and when it has data

After `tests/check.sh` passes, ask the learner to add one more task manually and re-run `list` to confirm the storage model makes sense.

## Builder Track

Implement a minimal `task_cli.py` that supports `add` and `list`, stores tasks in `tasks.json`, and satisfies all acceptance criteria in `spec.md`.

Execution rules:

- Run `tests/check.sh`
- If checks fail, fix and re-run until pass
- Keep commentary minimal unless user asks
