# Shared Command: `primer-explain`

Show or summarize the explanation for the current milestone.

## Inputs

- Context file with `primer_state`
- Recipe path from context file
- Current milestone `explanation.md`

## Behavior

1. Read and validate `primer_state`.
2. Resolve current milestone from `recipe.yaml`.
3. Read `explanation.md` for current milestone.
4. Present explanation content (verbatim or concise summary, adapter-defined).

## State Mutation

None. State is read-only for this command.

## Failure Modes

- Missing/malformed `primer_state`
- Unknown current milestone
- Missing `explanation.md`
