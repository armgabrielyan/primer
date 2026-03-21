# Primer

Primer is a learner-first recipe library for building substantial software projects with AI coding agents using milestone contracts, checks, and demos.

## Table of Contents

- [Start here (learners)](#start-here-learners)
- [Available recipes (projects)](#available-recipes-projects)
- [How to start any recipe](#how-to-start-any-recipe)
- [Milestone workflow (inside your AI tool)](#milestone-workflow-inside-your-ai-tool)
- [Repository layout](#repository-layout)
- [Contributing](#contributing)

## Start here (learners)

If you want to start a project, follow this flow:

1. Pick a recipe from **Available recipes** below.
2. Generate adapter files for your AI tool.
3. Open your project with that tool and follow milestone commands.
4. Use `check` at each milestone before moving forward.

## Available recipes (projects)

Current catalog:

| Recipe ID | Project | Difficulty | Estimated hours | Path |
|---|---|---|---:|---|
| `operating-system` | Build Your Own Operating System | `hard` | 40 | `recipes/operating-system` |

To list recipe folders directly:

```bash
find recipes -mindepth 1 -maxdepth 1 -type d -exec basename {} \;
```

## How to start any recipe

Set the recipe once:

```bash
RECIPE_ID=operating-system
```

Validate recipe contract:

```bash
scripts/validate-recipe "recipes/$RECIPE_ID"
```

Generate Claude Code adapter files:

```bash
scripts/generate-claude-adapter "recipes/$RECIPE_ID" --output-dir .
```

This creates:

- `CLAUDE.md`
- `.claude/commands/next-milestone.md`
- `.claude/commands/check.md`
- `.claude/commands/explain.md`
- `.claude/commands/status.md`

Generate Codex adapter files:

```bash
scripts/generate-codex-adapter "recipes/$RECIPE_ID" --output-dir .
```

This creates:

- `AGENTS.md`
- `.codex/next-milestone.md`
- `.codex/check.md`
- `.codex/explain.md`
- `.codex/status.md`

## Milestone workflow (inside your AI tool)

Use these commands/tasks milestone by milestone:

- `next-milestone`: verify current milestone and advance only on pass
- `check`: run current milestone verification
- `explain`: read the deep-dive explanation
- `status`: show current milestone and progress

## Repository layout

- `recipe-spec.md`: canonical recipe contract for v0.1
- `recipes/`: recipes and milestone contracts
- `adapters/_shared/`: shared command behavior + state model
- `scripts/`: validators and adapter generators
- `tests/`: recipe validation, shared command conformance, adapter generation tests

## Contributing

See `CONTRIBUTING.md` for contributor checks, quality gates, adapter standards, and test requirements.
