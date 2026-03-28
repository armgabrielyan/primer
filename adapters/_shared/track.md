# Shared Skill: `primer-track`

Switch the active workspace track between learner and builder.

## Inputs

- Context file with `primer_state`
- Requested target track from the user

## Behavior

1. Read and validate `primer_state`.
2. Parse the requested target track as either `learner` or `builder`.
3. If the target track is already active, report that no state change was needed.
4. Otherwise update `primer_state.track` to the requested track.
5. Keep `milestone_id` and `verified_milestone_id` unchanged.
6. Return the active track and the next recommended action.

## State Mutation

- Allowed: `primer_state.track`
- Forbidden: changes to `source`, `workspace_root`, `milestone_id`, `verified_milestone_id`, `stack_id`

## Failure Modes

- Missing/malformed `primer_state`
- Unsupported target track value
