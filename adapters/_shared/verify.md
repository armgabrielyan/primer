# Shared Skill: `primer-verify`

Run verification for the current milestone in the learner project workspace.

## Inputs

- Context file with `primer_state`
- Recipe path from context file
- Workspace root from context file
- Current milestone verification script

## Behavior

1. Read and validate `primer_state`.
2. Resolve current milestone from `recipe.yaml`.
3. Execute the current milestone verification script from `workspace_root`.
4. If it passes, set `primer_state.verified_milestone_id` to the current milestone id.
5. If it fails, clear `primer_state.verified_milestone_id` when it currently points at that milestone.
6. Return pass/fail with script output.

## State Mutation

- On success: `verified_milestone_id = milestone_id`
- On failure after a prior success for the same milestone: `verified_milestone_id = null`

## Failure Modes

- Missing/malformed `primer_state`
- Unknown current milestone
- Missing milestone verification script
- Script non-zero exit
