# Build a Task Tracker CLI

This recipe takes you from a runnable command-line script to a tested task tracker through 4 milestones.

## Table of Contents

- [What you will build](#what-you-will-build)
- [Prerequisites](#prerequisites)
- [Why this recipe is a good starting point](#why-this-recipe-is-a-good-starting-point)
- [How to start](#how-to-start)
- [Milestones](#milestones)
- [Recipe validation](#recipe-validation)

## What you will build

- a Python CLI entrypoint
- JSON-backed task storage
- commands to add, list, complete, edit, and remove tasks
- status filters and a summary command
- standard-library unit tests

## Prerequisites

- `python3`: used to run the CLI and unit tests
- `bash`: used by milestone checks

Notes:

- This recipe intentionally uses the Python standard library so setup stays simple.
- You do not need external packages such as `click`, `typer`, or `pytest`.

## Why this recipe is a good starting point

This recipe is meant to be a strong first Primer experience:

- the feedback loop is fast
- the project is practical and easy to understand
- each milestone adds one obvious capability
- the checks stay focused on visible behavior

You still learn useful habits here: parsing command-line input, persisting data safely, validating user input, and testing behavior instead of guessing.

## How to start

From a separate target workspace:

```bash
mkdir -p ~/workspace/task-cli
primer init cli-tool --tool codex --path ~/workspace/task-cli
cd ~/workspace/task-cli
primer doctor cli-tool --milestone 01-bootstrap
```

Then open that generated workspace in your AI tool and use:

- `primer-build`
- `primer-status`
- `primer-verify`
- `primer-explain`
- `primer-next-milestone`

Recommended rhythm:

1. Read the current milestone explanation and spec.
2. Run `primer-build` and implement only that milestone.
3. Run `primer-verify` until it passes.
4. Run `primer-next-milestone` only after the milestone is verified.

## Milestones

1. Bootstrap
2. Complete Tasks
3. Edit and Remove
4. Filters and Tests

## Recipe validation

`primer init` validates the bundled recipe contract before writing adapter files into the workspace.
