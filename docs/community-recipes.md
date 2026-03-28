# Community Recipes

Primer is meant to grow into a shared library of verified milestone paths.

If you have a project that is teachable in milestones, testable from the command line, and valuable for other learners, we want contributions.

Good community recipes are not just project templates. They are structured learning paths:

- each milestone has a narrow, visible goal
- each milestone has a clear verification step
- the learner can make progress in one real workspace over time
- the recipe teaches sequencing, not just file generation

## What To Contribute

The best recipe ideas tend to have all of these properties:

- substantial enough that an unguided prompt usually drifts or stalls
- small enough per milestone that an agent can stay focused
- runnable on a normal developer machine
- verifiable with deterministic shell commands
- educational for both learner and builder tracks

Examples of promising directions:

- shells
- databases
- compilers
- networking projects
- interpreters
- distributed systems labs
- graphics or game-engine subsystems

## Quality Bar

Before opening a PR, ask:

1. Does each milestone have one obvious success condition?
2. Can a learner recover if the agent makes a wrong turn?
3. Does the milestone verification script catch the most common failure modes?
4. Does the recipe teach something cumulative instead of bouncing between disconnected tasks?

A recipe is easier to review when:

- milestone scopes are crisp
- titles are concrete
- goals explain the capability change clearly
- verification summaries explain what passing actually proves
- demos describe user-visible outcomes
- prerequisites are honest and cumulative
- failure messages explain what to do next

## Authoring Rules

Start with these source documents:

- [recipe-spec.md](../recipe-spec.md)
- [CONTRIBUTING.md](../CONTRIBUTING.md)

Each recipe lives under `recipes/<recipe-id>/` and must follow the contract exactly.

Keep the design simple:

- one recipe should represent one learning path
- one milestone should unlock one meaningful concept or capability
- avoid speculative breadth in early-stage recipes
- prefer deterministic verification over clever verification

## Suggested Contribution Flow

1. Write the milestone list first.
2. Verify the ordering makes sense without reading implementation details.
3. Write `recipe.yaml` with realistic prerequisites and demos.
4. Draft `spec.md`, `agent.md`, and `explanation.md` for one milestone before writing all of them.
5. Add the milestone verification script and `demo.sh` as soon as possible so the contract stays honest.
6. Run `cargo test` before opening a PR.

## Review Expectations

Maintainers will look for:

- contract compliance
- quality of milestone boundaries
- clarity of learner guidance
- reliable verification behavior
- documentation that makes the path approachable

We are more likely to merge a narrow, well-tested learning path than a broad ambitious one with fuzzy milestones.

## Scope Advice

If your idea is large, start with the first solid slice.

For example, prefer:

- `bytecode-vm`

over:

- `build-a-complete-programming-language-toolchain`

The smaller framing is easier to review, easier to teach, and easier for contributors to finish.

## Community Direction

Primer should feel like a collection of serious guided labs that the community can trust.

That means contributions should optimize for:

- safety by default
- visible progress
- deterministic verification
- high teaching value
- compatibility with the CLI workflow

If you want help shaping a recipe before writing the full contribution, open an issue or draft PR with:

- the recipe idea
- the milestone list
- expected local prerequisites
- what the learner should be able to demo at the end
