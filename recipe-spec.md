# Recipe Specification (v0.1)

This document is the canonical contract for recipe structure in `primer` v0.1.

## Recipe Location

Each recipe must live at:

`recipes/<recipe-id>/`

Example:

`recipes/operating-system/`

## Required Files

Each recipe must contain:

- `recipe.yaml`
- `README.md`
- `milestones/`

Each declared milestone must contain:

- `spec.md`
- `agent.md`
- `explanation.md`
- `tests/check.sh`
- `demo.sh`

## `recipe.yaml` Contract

Required top-level keys:

- `id` (string, kebab-case)
- `title` (string)
- `description` (string)
- `difficulty` (`easy|medium|hard|expert`)
- `stack` (object)
- `tracks` (object)
- `milestones` (non-empty array)

### `stack`

Required keys:

- `id` (string, kebab-case preferred)
- `label` (string)
- `tools` (non-empty string array)

v0.1 supports only singular `stack`. `stacks` is out of scope.

### `tracks`

Required keys:

- `learner` (object with `description`)
- `builder` (object with `description`)

### `milestones`

Each milestone item requires:

- `id` (format `NN-name`, e.g. `01-bootloader`)
- `title` (string)
- `demo` (string)
- `prerequisites` (non-empty string array of executable/tool names required for that milestone)

Constraints:

- IDs must be unique.
- Milestones must be ordered by numeric `NN` prefix.
- `prerequisites` should be cumulative for the milestone. If milestone `03` still needs tools from milestone `02`, include them in milestone `03`.

## Naming Rules

- Recipe ID: `^[a-z0-9]+(?:-[a-z0-9]+)*$`
- Milestone ID: `^[0-9]{2}-[a-z0-9]+(?:-[a-z0-9]+)*$`
- Milestone folder name must exactly match milestone ID.

## Execution Model (v0.1)

Milestones are cumulative in one working tree. Milestone directories define the contract and verification; implementation code evolves through all milestones in the same workspace.

## Validation

Run:

```bash
scripts/validate-recipe recipes/<recipe-id>
```

This runs:
1. `recipe.yaml` schema validation
2. Milestone structure validation
