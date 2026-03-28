# Shared Skill: `primer-status`

Report current progress and next action.

## Inputs

- Context file with `primer_state`
- Workflow source path from context file

## Behavior

1. Read and validate `primer_state`.
2. Resolve milestone index from the active workflow source.
3. Return:
  - workflow state
  - workflow source kind and id
  - workspace root
  - track
  - stack id when present
  - current milestone id and title
  - whether the current milestone is currently verified
  - verification attempt count
  - failed attempt count and current failure streak when relevant
  - latest verification result if available
  - retry or stuck signal derived from recent verification history
  - completed count and total count
  - next milestone id (or completion message if final)
  - next allowed action, including stuck-aware guidance after repeated failures

## State Mutation

None. State is read-only for this skill.

## Failure Modes

- Missing/malformed `primer_state`
- Unknown current milestone
