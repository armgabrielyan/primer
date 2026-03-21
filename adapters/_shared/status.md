# Shared Command: `primer-status`

Report current progress and next action.

## Inputs

- Context file with `primer_state`
- Recipe path from context file

## Behavior

1. Read and validate `primer_state`.
2. Resolve milestone index from `recipe.yaml`.
3. Return:
  - recipe id
  - workspace root
  - track
  - stack id
  - current milestone id and title
  - whether current milestone is already verified
  - completed count and total count
  - next milestone id (or completion message if final)

## State Mutation

None. State is read-only for this command.

## Failure Modes

- Missing/malformed `primer_state`
- Unknown current milestone
