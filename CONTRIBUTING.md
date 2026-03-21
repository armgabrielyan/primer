# Contributing to Primer

This project is contract-driven. Changes are accepted only when schema, structure, and behavior remain deterministic and verifiable.

## Prerequisites

- Bash
- Python 3
- `PyYAML` available to Python runtime

## Core rules

1. Follow `recipe-spec.md` for recipe structure and schema.
2. Keep shared command behavior in `adapters/_shared/` as the source of truth.
3. Do not introduce adapter-specific behavior that contradicts shared command contracts.
4. Ensure tests remain green before opening a PR.

## Required checks before PR

Single command for all automated test suites:

```bash
make test
```

Full pre-PR check for a specific recipe:

```bash
make check RECIPE_ID=<recipe-id>
```

`make check` runs:

1. recipe validation
2. all automated test suites
3. milestone script bash syntax checks

If milestone runtime checks require extra dependencies for a specific recipe (for example an emulator), document them in that recipe's README.

## Recipe quality bar

A recipe is ready when:

- Every milestone includes:
  - `spec.md`
  - `agent.md` with both tracks
  - `explanation.md`
  - `tests/check.sh`
  - `demo.sh`
- Learner track asks at least one question per milestone.
- Check scripts produce clear, actionable failures.
- Demo scripts are runnable and aligned with milestone goals.

## Adapter changes

When changing adapter generation:

1. Update generator scripts in `scripts/`.
2. Keep outputs aligned with `adapters/_shared/*`.
3. Add or update tests in:
   - `tests/claude-adapter/`
   - `tests/codex-adapter/`

## Commit guidance

- Keep commits scoped (schema, adapter, recipe, tests).
- Include command output summary in PR description:
  - which commands were run
  - pass/fail status
  - known limitations
