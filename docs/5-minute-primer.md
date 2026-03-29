# 5 Minute Primer

This is the shortest reliable Primer walkthrough today.

Use it when you want to see the core product loop clearly:

1. initialize a workspace
2. inspect the current milestone
3. verify before the code exists and watch it fail
4. implement one bounded milestone
5. verify it successfully
6. advance to the next milestone

The walkthrough uses the `cli-tool` recipe because it has a fast verification loop, simple prerequisites, and obvious milestone boundaries.

If you want a recording after reading the steps, there is an existing asciinema capture here:

- <https://asciinema.org/a/E4NcqnYRDugeMXkJ>

## Before You Start

You need:

- `primer` installed
- `python3`
- `bash`
- one supported AI coding tool such as Codex or Claude Code

## 1. Initialize The Demo Workspace

```bash
primer init cli-tool --tool codex --track learner --path ~/projects/task-cli-demo
cd ~/projects/task-cli-demo
primer doctor cli-tool --milestone 01-bootstrap
```

What you should see:

- Primer creates a separate workspace
- Primer writes the tool-specific workflow files
- `primer doctor` confirms the first milestone prerequisites

## 2. Open The Workspace In Your AI Tool

Open `~/projects/task-cli-demo` in your AI coding tool.

Primer's main workflow now happens through the generated actions inside that workspace:

- `primer-status`
- `primer-build`
- `primer-verify`
- `primer-next-milestone`
- `primer-explain`

## 3. Inspect The Starting State

Run:

```text
primer-status
```

You should see:

- the current milestone is `01-bootstrap`
- the workflow is ready to build
- the milestone is not verified yet

This is the baseline. Primer knows exactly where you are and what counts as progress.

## 4. Verify Too Early On Purpose

Run:

```text
primer-verify
```

Expected outcome:

- verification fails
- Primer keeps you on the same milestone
- the failure is visible instead of silently advancing

This is the key product behavior. Primer does not confuse "work started" with "milestone complete."

## 5. Build Only The Current Milestone

Run:

```text
primer-build
```

Use the returned milestone contract and track guidance to implement only milestone `01-bootstrap`.

At the end of this milestone, the workspace should have the first small useful version of the CLI, including:

- `task_cli.py`
- `tasks.json`

The goal is not to finish the whole recipe. The goal is to make exactly one verified milestone pass.

## 6. Verify Again

Run:

```text
primer-verify
```

Expected outcome:

- verification passes
- Primer records the milestone as verified
- Primer tells you that `primer-next-milestone` is now safe to run

This is the trust boundary. Advancement is earned by a passing check, not by intention.

## 7. Advance Safely

Run:

```text
primer-next-milestone
primer-status
```

You should now see:

- the current milestone is `02-complete`
- the new current milestone is not verified yet
- the workflow is ready for the next bounded step

That is the whole Primer loop in one small project:

1. build one milestone
2. verify it
3. advance only after verification passes

## What This Demo Proves

After this walkthrough, you have seen Primer's core product behavior directly:

- the agent works against an explicit milestone contract
- failed verification does not count as progress
- successful verification opens the gate to advance
- progress is visible and durable across milestones

If you want to keep going, stay in the same workspace and continue the `cli-tool` recipe.

If you want a more ambitious follow-up, try `interpreter-mini` next.
