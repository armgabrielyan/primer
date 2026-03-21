# Shared Command: `primer-check`

Run verification for the current milestone in the learner project workspace.

## Inputs

- Context file with `primer_state`
- Recipe path from context file
- Workspace root from context file
- Current milestone `tests/check.sh`

## Behavior

1. Read and validate `primer_state`.
2. Resolve current milestone from `recipe.yaml`.
3. Execute `tests/check.sh` for current milestone from `workspace_root`.
4. If it passes, set `primer_state.verified_milestone_id` to the current milestone id.
5. Return pass/fail with script output.

## State Mutation

- On success only: `verified_milestone_id = milestone_id`

## Failure Modes

- Missing/malformed `primer_state`
- Unknown current milestone
- Missing `tests/check.sh`
- Script non-zero exit
