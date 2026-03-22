# Milestone 04: Filters and Tests

## Goal

Finish the CLI with filtered list output, a summary command, and automated tests.

## What you'll build

- `list --status pending`
- `list --status done`
- `summary`
- a `tests/` directory with at least one unit test module for the CLI or its core logic

## Acceptance criteria

- `python3 task_cli.py list --status pending` shows only pending tasks
- `python3 task_cli.py list --status done` shows only completed tasks
- `python3 task_cli.py summary` prints total, pending, and done counts
- `python3 -m unittest discover -s tests -p 'test_*.py'` exits with code `0`
- The test suite covers at least one real behavior path, not only import smoke tests

## Files to create or update in project workspace

- `task_cli.py`
- `tests/`

## Suggested implementation notes

- Keep filtered list behavior predictable and easy to explain
- Decide whether `summary` should print one line or multiple lines, but include all three counts
- Standard-library tests are enough here; do not add external dependencies

## Resources

- https://docs.python.org/3/library/unittest.html
- https://docs.python.org/3/library/subprocess.html
