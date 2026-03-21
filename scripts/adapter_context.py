#!/usr/bin/env python3
"""Shared context renderer for adapter root instruction files."""

from __future__ import annotations

from pathlib import Path


def render_adapter_context(
    *,
    recipe_title: str,
    recipe_id: str,
    recipe_path: Path,
    workspace_root: Path,
    milestone_id: str,
    track: str,
    stack_id: str,
) -> str:
    return f"""# Primer — {recipe_title}

```yaml
primer_state:
  recipe_id: {recipe_id}
  recipe_path: {recipe_path.as_posix()}
  workspace_root: {workspace_root.as_posix()}
  milestone_id: {milestone_id}
  verified_milestone_id: null
  track: {track}
  stack_id: {stack_id}
```

## Recipe location

{recipe_path.as_posix()}/

## Workspace root

{workspace_root.as_posix()}/

## Rules

- Always read the current milestone `agent.md` before starting work.
- Work in this project workspace, not in the `primer` repository.
- Build the current milestone in small steps and do not implement future milestones early.
- Run current milestone `tests/check.sh` before declaring completion.
- Only run `primer-next-milestone` after `primer-check` has marked the current milestone as verified.
- Use the generated Primer workflow actions for behavior rules.

## Available workflow actions

- `primer-build` — implement the current milestone step by step
- `primer-next-milestone` — advance state only after the milestone is already verified
- `primer-check` — run current milestone checks
- `primer-explain` — show current milestone explanation
- `primer-status` — show current state and progress
"""
