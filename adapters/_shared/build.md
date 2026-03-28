# Shared Skill: `primer-build`

Build only the current milestone's required work, step by step.

## Inputs

- Context file with `primer_state`
- Workflow source path from context file
- Workspace root from context file
- Current milestone contract file `spec.md`
- Current milestone `agent.md`

## Behavior

1. Read and validate `primer_state`.
2. Resolve current milestone from the active workflow source.
3. Read the current milestone contract file `spec.md` and `agent.md`.
4. Build only the current milestone scope in `workspace_root`.
5. Do not implement future milestones or pre-build later subsystems.
6. In learner track:
   - explain each small step before changing code
   - pause at natural checkpoints
   - confirm understanding before moving to the next sub-step
7. In builder track:
   - implement the smallest complete change that moves the milestone forward
8. Recommend running `primer-verify` once the build step is complete.

## State Mutation

None. State is read-only for this skill.

## Reproducibility Rules

- Prefer deterministic file names, commands, and outputs from the milestone contract.
- Avoid opportunistic refactors outside the milestone scope.
- Keep the implementation aligned with the files listed in the milestone contract `spec.md` unless there is a concrete blocker.
