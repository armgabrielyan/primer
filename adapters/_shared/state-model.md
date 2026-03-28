# Primer State Model

All adapters must embed a machine-readable state block in their context file.

## Canonical Block

Recipe-backed workspace:

```yaml
primer_state:
  schema_version: 2
  source:
    kind: recipe
    id: operating-system
    path: /abs/path/to/workspace/.primer/recipes/operating-system
  workspace_root: /abs/path/to/my-os
  milestone_id: 01-bootloader
  verified_milestone_id: null
  track: learner
  stack_id: c-x86
```

Workstream-backed repository:

```yaml
primer_state:
  schema_version: 2
  source:
    kind: workstream
    id: auth-refactor
    path: /abs/path/to/repo/.primer/workstreams/auth-refactor
  workspace_root: /abs/path/to/repo
  milestone_id: 01-customize-first-milestone
  verified_milestone_id: null
  track: builder
```

## Field Rules

- `schema_version`: required, currently `2`
- `source.kind`: required, either `recipe` or `workstream`
- `source.id`: required workflow identifier
- `source.path`: required absolute path to the active workflow source
- `workspace_root`: required absolute path to the directory where implementation changes happen
- `milestone_id`: required, mutable via `next-milestone` only
- `verified_milestone_id`: optional, set by `verify` when current milestone passes
- `track`: required, mutable via `track`
- `stack_id`: optional, recipe-specific metadata

## Read Rules

- Commands must read `primer_state` from the context file.
- If the block is missing or malformed, the command fails with an explicit state error.
- `source.path` and `workspace_root` must be absolute paths.
- If `milestone_id` is not declared in the active workflow source, the command fails.
- If `verified_milestone_id` is present, it must be either `null` or a declared milestone id.

## Write Rules

- `verify` may set `verified_milestone_id` to the current milestone when verification passes.
- `track` may update `track` while keeping source, milestone, and verification state unchanged.
- `next-milestone` may update `milestone_id` and clear `verified_milestone_id`.
- `build`, `explain`, and `status` must not mutate state.
- If the current milestone is final, `next-milestone` must not mutate state.

## Determinism

Given the same state, workflow source, and command outcome, state transitions must be deterministic.
