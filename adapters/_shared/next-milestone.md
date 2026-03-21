# Shared Skill: `primer-next-milestone`

Advance to the next milestone only after the current milestone has already been verified.

## Inputs

- Context file with `primer_state`
- Recipe path from context file
- Workspace root from context file

## Behavior

1. Read and validate `primer_state`.
2. Resolve current milestone from `recipe.yaml`.
3. If `verified_milestone_id` is not equal to the current milestone id, stop and instruct the user to run `primer-check`.
4. If current milestone is final, return completion summary; do not update state.
5. Otherwise set `primer_state.milestone_id` to the next declared milestone and clear `verified_milestone_id`.
6. Load next milestone `spec.md` and `agent.md`.
7. Follow track behavior:
  - learner: introduce and ask required learner question(s)
  - builder: begin implementation directly

## State Mutation

- Allowed: `primer_state.milestone_id`
- Allowed: clear `primer_state.verified_milestone_id` when advancing
- Forbidden: changes to `recipe_id`, `track`, `stack_id`

## Failure Modes

- Missing/malformed `primer_state`
- Unknown current milestone
- current milestone not yet verified
