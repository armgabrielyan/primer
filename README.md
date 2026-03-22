# Primer

Primer is a toolkit for learning and building substantial software projects with AI coding agents, one milestone at a time.

You pick a recipe, initialize a real project workspace, and then move through a tight loop:

1. read the current milestone
2. build only that scope
3. run verification
4. advance only after it passes

The current first-class recipe is `operating-system`: build your own x86 operating system from bootloader to shell.

## Table of Contents

- [Why Primer](#why-primer)
- [Installation](#-installation)
- [Tool Integration](#-tool-integration)
- [Quickstart](#-quickstart)
- [First Run](#-first-run)
- [Core Commands](#-core-commands)
- [Available Recipe](#available-recipe)
- [Tracks](#tracks)
- [Prerequisites](#prerequisites)
- [Workspace Model](#workspace-model)
- [Example Flow](#example-flow)
- [Repository Layout](#repository-layout)
- [Contributing](#-contributing)

## Why Primer

Most AI coding workflows break down when the task is too large, too vague, or too easy to drift from.

Primer fixes that by giving the agent:

- a concrete milestone contract
- a verification script for the current step
- a clear notion of what comes next
- a learner track and a builder track

The result is a workflow that feels closer to a guided lab than an open-ended prompt.

Primer is also meant to become a community recipe library. If you want to contribute a new learning path, see [docs/community-recipes.md](docs/community-recipes.md).

## 📦 Installation

### Quick install (macOS/Linux)

```bash
curl -sSf https://raw.githubusercontent.com/armgabrielyan/primer/main/install.sh | sh
```

### Homebrew (macOS/Linux)

```bash
brew install armgabrielyan/tap/primer
```

### npm/npx

```bash
npm install -g @armengabrielyan/primer
```

This installs the `primer` command. For one-off usage:

```bash
npx @armengabrielyan/primer list
```

The `@armengabrielyan/primer` npm package downloads the matching prebuilt `primer` binary for your platform during install.

### Cargo

```bash
cargo install primer
```

### Native binaries

Download prebuilt archives from the [GitHub Releases](https://github.com/armgabrielyan/primer/releases) page.

| Platform | Architecture | Download |
|---|---|---|
| Linux | x86_64 (glibc) | `primer-VERSION-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | x86_64 (musl/static) | `primer-VERSION-x86_64-unknown-linux-musl.tar.gz` |
| Linux | ARM64 | `primer-VERSION-aarch64-unknown-linux-gnu.tar.gz` |
| macOS | Intel | `primer-VERSION-x86_64-apple-darwin.tar.gz` |
| macOS | Apple Silicon | `primer-VERSION-aarch64-apple-darwin.tar.gz` |
| Windows | x86_64 | `primer-VERSION-x86_64-pc-windows-msvc.zip` |

### Build from source

```bash
git clone https://github.com/armgabrielyan/primer
cd primer
cargo build --release
```

The binary will be available at:

```bash
./target/release/primer
```

### Install from local source

```bash
cargo install --path .
```

This installs `primer` into Cargo's bin directory so you can run it directly:

```bash
primer list
```

### Shell completions

Primer can generate shell completions for bash, zsh, and fish:

```bash
primer completions zsh
primer completions bash
primer completions fish
```

## 🤖 Tool Integration

Primer currently supports four AI coding tools:

- OpenCode
- Gemini CLI
- Claude Code
- Codex

`primer init` generates tool-specific files into the workspace so the workflow is available where the project work actually happens.

### OpenCode

Use:

```bash
primer init <recipe-id> --tool opencode --path ~/projects/my-workspace
```

Primer generates:

- `AGENTS.md`
- `.opencode/skills/`

These skills are loaded by OpenCode's native skill system, while `AGENTS.md` provides the project rules and Primer workflow context for the workspace.

### Gemini CLI

Use:

```bash
primer init <recipe-id> --tool gemini --path ~/projects/my-workspace
```

Primer generates:

- `GEMINI.md`
- `.gemini/skills/`

Gemini CLI loads `GEMINI.md` as project context and discovers the generated Primer skills from `.gemini/skills/`.

### Claude Code

Use:

```bash
primer init <recipe-id> --tool claude --path ~/projects/my-workspace
```

Primer generates:

- `CLAUDE.md`
- `.claude/commands/`

These commands are thin wrappers around the Primer workflow, with the CLI acting as the source of truth for stateful actions such as `primer-check`, `primer-status`, and `primer-next-milestone`.

### Codex

Use:

```bash
primer init <recipe-id> --tool codex --path ~/projects/my-workspace
```

Primer generates:

- `AGENTS.md`
- `.agents/skills/`

These skills expose the same Primer workflow inside Codex, including `primer-build`, `primer-check`, `primer-explain`, `primer-status`, and `primer-next-milestone`.

## 🚀 Quickstart

The installed CLI includes the built-in recipe catalog, so you do not need to keep a cloned `primer` repo around just to use `primer list`, `primer init`, or `primer doctor`.

List available recipes:

```bash
primer list
```

Create a workspace for the operating-system recipe:

```bash
primer init operating-system --tool claude --path ~/projects/my-os
```

Check local dependencies for the first milestone:

```bash
primer doctor operating-system --milestone 01-bootloader
```

Open the generated workspace in your AI tool, then use the Primer skills inside that workspace:

- `primer-build`: load the current milestone contract and active track guidance, then work only that scope
- `primer-check`: run verification for the current milestone and mark it verified on success
- `primer-next-milestone`: advance only after the current milestone has already passed verification
- `primer-explain`: show the deeper explanation for the current milestone
- `primer-status`: show current milestone, verification state, and progress

If you prefer, you can also use the CLI directly from the workspace. The command surface looks like this:

```text
Usage: primer [OPTIONS] <COMMAND>

Commands:
  list            List available recipes
  init            Initialize a new Primer workspace
  doctor          Check required local tools for a recipe milestone
  status          Show current Primer workspace progress
  check           Run verification for the current milestone
  next-milestone  Advance to the next milestone after verification
  explain         Show the explanation for the current milestone
  build           Show current milestone build guidance
  completions     Generate shell completion scripts
```

## 🧭 First Run

The fastest path from zero to working milestone looks like this:

1. Run `primer init` to create a separate learner workspace.
2. Run `primer doctor` to see what tools are missing before you start.
3. Open that workspace in OpenCode, Gemini CLI, Claude Code, or Codex.
4. Run the `primer-build` skill to load the current milestone contract and start implementing.
5. Run the `primer-check` skill when you think the milestone is done.
6. Run the `primer-next-milestone` skill only after the check passes.

Primer creates a real project workspace. That workspace is where `Makefile`, source files, linker scripts, and milestone outputs live. The `primer` repository stays clean and acts as the recipe library plus workflow engine.

## 🛠️ Core Commands

The current CLI surface is:

- `primer list`: show available recipes
- `primer init`: create a workspace and generate adapter files
- `primer doctor`: check milestone prerequisites
- `primer status`: show current milestone, verification state, and progress
- `primer build`: show the current milestone spec and active track guidance
- `primer check`: run current milestone verification and update state on success
- `primer next-milestone`: advance one milestone and clear verification state
- `primer explain`: show the current milestone explanation
- `primer completions`: generate shell completions

The CLI is the source of truth for deterministic workflow actions:

- `primer-check`
- `primer-status`
- `primer-explain`
- `primer-next-milestone`

Generated OpenCode, Gemini, Claude, and Codex skills call into the CLI for those actions. `primer-build` stays agent-native, but uses `primer build` to load the current milestone contract first.

## Available Recipe

Current catalog:

| Recipe ID | Project | Difficulty | Path |
|---|---|---|---|
| `operating-system` | Build Your Own Operating System | `hard` | `recipes/operating-system` |

## Tracks

Primer supports two tracks for each milestone:

- `learner`: explain the step, ask at least one question, and teach while building
- `builder`: implement directly with minimal commentary

Choose the track at workspace initialization:

```bash
primer init operating-system --tool codex --track learner --path ~/projects/my-os
```

## Prerequisites

For the current `operating-system` recipe, the important tools are:

- `nasm`: required from milestone 01
- `qemu-system-i386`: required to run and verify the OS image
- `make`: required in the learner project workspace
- `i686-elf-gcc` and `i686-elf-ld`: required from milestone 02 onward

Use `primer doctor` instead of guessing:

```bash
primer doctor operating-system --milestone 02-kernel-entry
```

Different recipes can declare different prerequisites. The CLI reads those requirements from the recipe milestones, so treat the list above as recipe-specific, not global.

## Workspace Model

Primer uses two separate locations:

- the `primer` repo: recipes, shared contracts, adapter generation, CLI
- your project workspace: the actual code you are building milestone by milestone

Do not build inside the `primer` repo itself. `primer init` is designed to create or prepare a separate workspace for exactly this reason.

## Example Flow

Here is the intended day-to-day loop from inside a workspace:

1. Run `primer-status` to see where you are.
2. Run `primer-build` to load the current milestone contract and work only that scope.
3. Run `primer-check` to verify the result.
4. Run `primer-explain` if you want the deeper reasoning behind the milestone.
5. Run `primer-next-milestone` to unlock the next step.

This gives the agent a constrained problem and gives you a visible notion of progress.

## Repository Layout

- `src/`: Rust CLI implementation
- `recipes/`: milestone contracts and recipe content
- `adapters/_shared/`: shared skill behavior and state model
- `tests/`: Rust CLI and bundled-workflow tests
- `recipe-spec.md`: canonical recipe contract for v0.1

## 🤝 Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for contributor checks, quality gates, adapter standards, and test requirements.

If you want to propose a new community learning path, start with [docs/community-recipes.md](docs/community-recipes.md).
