# Contributing to Primer

This project is contract-driven. Changes are accepted only when schema, structure, and behavior remain deterministic and verifiable.

Primer is also intended to become a community library of learning paths. If you want to contribute a new recipe, start with [docs/community-recipes.md](docs/community-recipes.md).

## Prerequisites

- Bash
- Rust toolchain
- Cargo

## Core rules

1. Follow `recipe-spec.md` for recipe structure and schema.
2. Keep shared command behavior in `adapters/_shared/` as the source of truth.
3. Do not introduce adapter-specific behavior that contradicts shared command contracts.
4. Ensure tests remain green before opening a PR.

## Required checks before PR

Run the automated test suite:

```bash
cargo test
```

For recipe contributions, also review the recipe's own `README.md` and milestone scripts to make sure the documented prerequisites and checks still match reality.

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

1. Update the Rust generator logic in `src/adapter.rs`.
2. Keep outputs aligned with `adapters/_shared/*`.
3. Add or update Rust tests for generated outputs.

## Commit guidance

- Keep commits scoped (schema, adapter, recipe, tests).
- Include command output summary in PR description:
  - which commands were run
  - pass/fail status
  - known limitations
