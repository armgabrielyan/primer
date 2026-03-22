# Milestone 03: Edit and Remove

## Goal

Extend the task lifecycle so tasks can be edited and removed by id.

## What you'll build

- an `edit` command that replaces a task title by id
- a `remove` command that deletes a task by id
- consistent missing-id error handling across update commands

## Acceptance criteria

- `python3 task_cli.py edit 2 "read the Primer docs"` updates task `2`
- `python3 task_cli.py remove 1` removes task `1`
- `tasks.json` still contains valid JSON after both operations
- The remaining tasks keep their ids and updated titles
- `python3 task_cli.py edit 99 "x"` exits non-zero and prints a helpful error message
- `python3 task_cli.py remove 99` exits non-zero and prints a helpful error message

## Files to update in project workspace

- `task_cli.py`

## Suggested implementation notes

- Keep command behavior consistent across `done`, `edit`, and `remove`
- Decide whether task ids are never reused or only increment from the current max id
- Make command names and help output explicit enough for someone else to discover

## Resources

- https://docs.python.org/3/library/argparse.html#sub-commands
