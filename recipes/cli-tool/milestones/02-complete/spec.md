# Milestone 02: Complete Tasks

## Goal

Extend the CLI so tasks can be marked done by id and listed with visible status.

## What you'll build

- a `done` command that marks an existing task complete
- stable integer ids for tasks
- list output that clearly distinguishes pending tasks from completed tasks

## Acceptance criteria

- `python3 task_cli.py add "buy milk"` and `python3 task_cli.py add "read docs"` create two tasks
- `python3 task_cli.py done 1` marks the first task as complete
- `tasks.json` keeps the same ids and titles while changing only the `done` state
- `python3 task_cli.py list` shows both tasks and makes the done versus pending state visible
- `python3 task_cli.py done 99` exits non-zero and prints a helpful error message

## Files to update in project workspace

- `task_cli.py`

## Suggested implementation notes

- Look up tasks by id, not by position in the array
- Return a non-zero exit code when the requested id does not exist
- Keep the storage format from milestone 01 unchanged

## Resources

- https://docs.python.org/3/library/sys.html
- https://docs.python.org/3/library/argparse.html
