# Build a Mini Expression Interpreter

This recipe takes you from tokenizing source text to running a tiny assignment-based language through 4 milestones.

## Table of Contents

- [What you will build](#what-you-will-build)
- [Prerequisites](#prerequisites)
- [Why this recipe is intermediate](#why-this-recipe-is-intermediate)
- [How to start](#how-to-start)
- [Milestones](#milestones)
- [Recipe validation](#recipe-validation)

## What you will build

- a tokenizer for a tiny expression language
- a parser that produces a structured AST
- an evaluator for arithmetic expressions
- variable assignment support from a source file
- standard-library unit tests

## Prerequisites

- `python3`: used to run the interpreter and unit tests
- `bash`: used by milestone checks

Notes:

- This recipe intentionally stays inside the Python standard library.
- The focus is language implementation fundamentals, not framework usage.

## Why this recipe is intermediate

This recipe is more conceptual than `cli-tool`:

- you are designing a language model, not just command behavior
- each milestone builds on earlier representation choices
- parser and evaluator bugs often show up as logic errors rather than obvious crashes

That makes it a good next step after a practical beginner project. It is still bounded enough to finish, but it asks you to reason about structure, precedence, and state.

## How to start

From a separate target workspace:

```bash
mkdir -p ~/workspace/interpreter-mini
primer init interpreter-mini --tool codex --path ~/workspace/interpreter-mini
cd ~/workspace/interpreter-mini
primer doctor interpreter-mini --milestone 01-tokenizer
```

Then open that generated workspace in your AI tool and use:

- `primer-build`
- `primer-status`
- `primer-check`
- `primer-explain`
- `primer-next-milestone`

Recommended rhythm:

1. Read the current milestone explanation and spec.
2. Run `primer-build` and implement only that milestone.
3. Run `primer-check` until it passes.
4. Run `primer-next-milestone` only after the milestone is verified.

## Milestones

1. Tokenizer
2. Parser
3. Evaluator
4. Variables and Tests

## Recipe validation

`primer init` validates the bundled recipe contract before writing adapter files into the workspace.
