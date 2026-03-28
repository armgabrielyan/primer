# Shared Skill: `primer-status`

Report current progress and next action.

## Inputs

- Context file with `primer_state`
- Recipe path from context file

## Behavior

1. Read and validate `primer_state`.
2. Resolve milestone index from `recipe.yaml`.
3. Return:
  - workflow state
  - recipe id
  - workspace root
  - track
  - stack id
  - current milestone id and title
  - whether current milestone is already verified
  - verification attempt count
  - latest verification result if available
  - completed count and total count
  - next milestone id (or completion message if final)
  - next allowed action

## State Mutation

None. State is read-only for this skill.

## Failure Modes

- Missing/malformed `primer_state`
- Unknown current milestone
