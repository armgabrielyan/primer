# Milestone 01: Bootstrap

## Goal

Build a runnable Python task tracker CLI that can add tasks to `tasks.json` and list them back.

## What you'll build

- `task_cli.py`: the main command-line entrypoint
- `tasks.json`: a JSON storage file created by the program when needed

Use this task shape in `tasks.json`:

```json
{
  "id": 1,
  "title": "buy milk",
  "done": false
}
```

## Acceptance criteria

- `python3 task_cli.py --help` exits with code `0`
- `python3 task_cli.py list` prints a clear "no tasks" message when storage is missing or empty
- `python3 task_cli.py add "buy milk"` creates `tasks.json`
- `tasks.json` is valid JSON and contains a list of task objects
- The first task has:
  - `id` equal to `1`
  - `title` equal to `"buy milk"`
  - `done` equal to `false`
- `python3 task_cli.py list` shows the task id and title

## Files to create in project workspace

- `task_cli.py`

## Suggested implementation notes

- Use `argparse` for command parsing
- Use `json` and `pathlib` from the standard library
- Keep storage logic simple and explicit
- If `tasks.json` does not exist yet, treat it as an empty task list

## Resources

- https://docs.python.org/3/library/argparse.html
- https://docs.python.org/3/library/json.html
- https://docs.python.org/3/library/pathlib.html
