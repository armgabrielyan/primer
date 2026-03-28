# Primer State Model

All adapters must embed a machine-readable state block in their context file.

## Canonical Block

```yaml
primer_state:
  recipe_id: operating-system
  recipe_path: /abs/path/to/primer/recipes/operating-system
  workspace_root: /abs/path/to/my-os
  milestone_id: 01-bootloader
  verified_milestone_id: null
  track: learner
  stack_id: c-x86
```

## Field Rules

- `recipe_id`: required, immutable during a recipe run
- `recipe_path`: required absolute path to the recipe in the `primer` repo
- `workspace_root`: required absolute path to the learner project workspace
- `milestone_id`: required, mutable via `next-milestone` only
- `verified_milestone_id`: optional, set by `verify` when current milestone passes
- `track`: required, immutable for the current command flow
- `stack_id`: required, immutable for the current command flow

## Read Rules

- Commands must read `primer_state` from the context file.
- If block is missing or malformed, command fails with explicit state error.
- `recipe_path` and `workspace_root` must be absolute paths.
- If `milestone_id` is not declared in recipe milestones, command fails.
- If `verified_milestone_id` is present, it must be either `null` or a declared milestone id.

## Write Rules

- `verify` may set `verified_milestone_id` to the current milestone when verification passes.
- `next-milestone` may update `milestone_id` and clear `verified_milestone_id`.
- `build`, `explain`, and `status` must not mutate state.
- If current milestone is final, `next-milestone` must not mutate state.

## Determinism

Given the same state, recipe, and command outcome (milestone verification success), state transitions must be deterministic.
