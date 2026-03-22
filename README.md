# Primer

Primer is a guided workflow for learning and building real software projects with AI coding agents, one milestone at a time.

The main work happens inside your AI coding agent. The `primer` CLI sets up the workspace, checks prerequisites, and powers the workflow under the hood.

Instead of handing an agent a huge vague task, Primer gives you a smaller contract, a way to verify it, and a clear next step.

Primer is a good fit if you want to:

- start from a real workspace instead of a blank prompt
- keep the agent focused on the current step
- verify progress before moving on
- learn the system as you build it

Current recipes:

- `cli-tool`: build a practical task tracker CLI in Python
- `interpreter-mini`: build a small expression language in Python
- `operating-system`: build an x86 operating system from bootloader to shell

## Table of Contents

- [Start Here](#start-here)
- [How Primer Works](#how-primer-works)
- [Why Primer](#why-primer)
- [Is Primer Beginner-Friendly?](#is-primer-beginner-friendly)
- [Who It's For](#who-its-for)
- [Installation](#installation)
  - [Recommended quick install (macOS/Linux)](#recommended-quick-install-macoslinux)
  - [Homebrew (macOS/Linux)](#homebrew-macoslinux)
  - [npm / npx](#npm--npx)
  - [Cargo](#cargo)
  - [Native binaries](#native-binaries)
  - [Build from source](#build-from-source)
  - [Install from local source](#install-from-local-source)
  - [Sanity check after install](#sanity-check-after-install)
  - [Shell completions](#shell-completions)
- [AI Tool Integration](#ai-tool-integration)
- [Available Recipe](#available-recipe)
- [Tracks](#tracks)
- [Agent Workflow Actions](#agent-workflow-actions)
- [CLI Setup Commands](#cli-setup-commands)
- [Workspace Model](#workspace-model)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)
- [License](#license)

## Start Here

If you want the fastest path from zero to building inside your AI coding agent:

```bash
primer list
primer init operating-system --tool codex --track learner --path ~/projects/my-os
cd ~/projects/my-os
primer doctor operating-system --milestone 01-bootloader
```

Use the terminal for setup:

1. `primer list` shows the catalog.
2. `primer init` creates a separate project workspace and generates AI-tool instructions for it.
3. `primer doctor` checks whether your local toolchain is ready for the first milestone.

Then open the generated workspace in your AI coding agent and use the Primer workflow actions there:

- `primer-build`: implement only the current milestone scope
- `primer-check`: run milestone verification and mark it verified on success
- `primer-next-milestone`: unlock the next milestone only after verification passes
- `primer-explain`: show the deeper explanation for the current milestone
- `primer-status`: show current milestone, verification state, and progress

Primer is designed so that:

- In a regular shell, you mainly use `primer` for setup, diagnostics, and utilities such as `init`, `doctor`, and `completions`.
- Inside a generated workspace in a supported AI tool, you do the actual milestone work through generated actions such as `primer-build` and `primer-status`.

If you are new to Primer, start with `--track learner`.

## How Primer Works

Primer has two layers:

1. The setup layer: the `primer` CLI creates the workspace, checks your environment, and manages workflow state.
2. The working layer: your AI coding agent uses the generated Primer actions inside that workspace while you build milestone by milestone.

In practice, that means:

- you touch the terminal first to initialize the workspace
- you spend most of your time inside the AI coding agent
- Primer verification decides when a milestone is actually complete

The CLI is important infrastructure, but it is not meant to be the primary day-to-day interface.

## Why Primer

Most AI coding workflows fail in predictable ways:

- the task is too broad
- the agent implements future steps too early
- there is no reliable check for "done"
- progress is hard to see

Primer turns that into a repeatable loop:

1. Initialize a separate project workspace from a recipe.
2. Load the current milestone contract.
3. Build only that scope.
4. Run verification.
5. Advance only after it passes.

The result feels closer to a guided lab than an open-ended prompt.

## Is Primer Beginner-Friendly?

The workflow is beginner-friendly. The current recipe catalog is not broadly beginner-oriented yet.

That distinction matters:

- Primer itself is designed to be approachable for students, entry-level developers, and people learning with AI assistance.
- The current flagship recipe, `operating-system`, is still an advanced systems project with a real toolchain and low-level concepts.

If you are a beginner or novice, the recommended path is:

- use `--track learner`
- start with the `cli-tool` recipe
- run `primer doctor` early so tooling issues are obvious
- use `primer explain` and `primer check` as part of every milestone
- expect to learn incrementally instead of understanding the entire system up front

If you are completely new to programming, `cli-tool` is the best place to start. The more ambitious `operating-system` recipe is better treated as an advanced guided lab.

## Who It's For

Primer is for people who want structure while building with AI tools:

- learners who want explanations, checkpoints, and visible progress
- builders who want tighter scope control and safer iteration
- educators and recipe authors who want reusable milestone-based learning paths

If you want to try ambitious projects without handing the whole problem to the agent at once, Primer is the point.

Primer is also meant to grow into a community library of guided labs. If you want to help create new recipes or improve the educational experience around existing ones, see [CONTRIBUTING.md](CONTRIBUTING.md) and [docs/community-recipes.md](docs/community-recipes.md).

## Installation

Pick one installation path. You install the `primer` CLI because it bootstraps and supports the agent workflow.

### Recommended quick install (macOS/Linux)

```bash
curl -sSf https://raw.githubusercontent.com/armgabrielyan/primer/main/install.sh | sh
```

### Homebrew (macOS/Linux)

```bash
brew install armgabrielyan/tap/primer
```

### npm / npx

```bash
npm install -g @armengabrielyan/primer
```

For one-off usage:

```bash
npx @armengabrielyan/primer list
```

The `@armengabrielyan/primer` package downloads the matching prebuilt `primer` binary for your platform during install.

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

### Sanity check after install

```bash
primer --help
primer list
```

### Shell completions

Primer can generate shell completions for bash, zsh, and fish:

```bash
primer completions zsh
primer completions bash
primer completions fish
```

## AI Tool Integration

Primer has native workspace adapters for:

- Claude Code
- Codex
- OpenCode
- Gemini CLI
- Cursor

Primer is designed to be used inside your AI coding agent. `primer init` generates tool-specific files into the workspace so the workflow is available where the project work actually happens.

Use the same `primer init` shape for every supported tool:

```bash
primer init <recipe-id> --tool <claude|codex|opencode|gemini|cursor> --path ~/projects/my-workspace
```

Generated files by tool:

| Tool | Generated files |
|---|---|
| Claude Code | `CLAUDE.md`, `.claude/commands/` |
| Codex | `AGENTS.md`, `.agents/skills/` |
| OpenCode | `AGENTS.md`, `.opencode/skills/` |
| Gemini CLI | `GEMINI.md`, `.gemini/skills/` |
| Cursor | `AGENTS.md`, `.cursor/skills/` |

Those generated actions are the primary user experience. The CLI provides the underlying stateful operations, verification, and setup support that those actions rely on.

If your preferred AI tool can follow workspace instructions and run the local `primer` CLI, Primer can usually fit that workflow as well.

## Available Recipe

Current catalog:

| Recipe ID | Project | Difficulty | Best starting advice |
|---|---|---|---|
| `cli-tool` | Build a Task Tracker CLI | `beginner` | Start here if you are new to Primer, new to AI-assisted building, or want a fast first success |
| `interpreter-mini` | Build a Mini Expression Interpreter | `intermediate` | Start here after `cli-tool` if you want a more conceptual project with parsing and evaluation |
| `operating-system` | Build Your Own Operating System | `advanced` | Start with `--track learner` and treat it like a guided lab, not a quick tutorial |

For more detail on the current recipe, see [recipes/operating-system/README.md](recipes/operating-system/README.md).

## Tracks

Primer supports two interaction styles:

| Track | What it feels like | Best for |
|---|---|---|
| `learner` | The agent explains the step, teaches while building, and pauses at natural checkpoints | beginners, students, and first-time Primer users |
| `builder` | The agent implements directly with minimal commentary | users who already understand the workflow and want less narration |

Examples:

```bash
primer init operating-system --tool codex --track learner --path ~/projects/my-os
```

```bash
primer init operating-system --tool codex --track builder --path ~/projects/my-os
```

If you do not pass `--track`, Primer uses `learner`.

## Agent Workflow Actions

Once the workspace is initialized, this is the primary way to use Primer:

| Action | What it does | When to use it |
|---|---|---|
| `primer-build` | Load the current milestone scope and implement only that step | when you are actively building |
| `primer-status` | Show current milestone, verification state, and progress | anytime you want orientation |
| `primer-explain` | Show the deeper explanation for the current milestone | when you want more context or teaching |
| `primer-check` | Run milestone verification and mark it verified on success | when you think the milestone is done |
| `primer-next-milestone` | Unlock the next milestone only after verification passes | when you are ready to advance |

Primer also exposes matching CLI commands such as `primer build`, `primer status`, `primer explain`, `primer check`, and `primer next-milestone`, but the default experience is to use the generated actions inside your AI coding agent.

## CLI Setup Commands

Use the CLI directly for setup, diagnostics, and terminal utilities:

| Command | What it does | When to use it |
|---|---|---|
| `primer list` | List available recipes | when you are exploring |
| `primer init` | Create a workspace and generate adapter files | when you are starting a new project |
| `primer doctor` | Check local prerequisites for a recipe milestone | before you begin or when setup is failing |
| `primer completions` | Generate shell completion scripts | when you want faster terminal use |

Useful safety flags:

- `primer init --dry-run` shows what would happen without writing files.
- `primer init --force` allows initialization into a non-empty directory.

## Workspace Model

Primer uses two separate locations:

- the `primer` repo: recipes, adapter generation, the CLI engine, and shared workflow logic
- your generated project workspace: the code and Primer state for the project you are building
- your AI coding agent: the main place where you read, build, check, and advance milestones

Do not build inside the `primer` repo itself. `primer init` is designed to create or prepare a separate workspace for that work.

After `primer init`, your generated workspace contains:

- the project files your agent will actually build
- the current Primer state
- tool-specific instructions and workflow actions

For most users, the generated workspace opened inside the AI coding agent is the primary interface. The CLI exists to support that workflow.

## Troubleshooting

- Run `primer doctor` after `primer init` if you are unsure whether your local toolchain is ready for the current milestone.
- If your AI tool does not see the generated Primer workflow actions, make sure you opened the generated workspace, not the `primer` repository.
- If your AI tool cannot run `primer`, install or build the CLI first and make sure it is on your `PATH`.
- `primer init` is safe by default. Use `--force` only when you deliberately want to initialize into a non-empty directory.

## Contributing

Primer is not only for people using recipes. It is also for people who want to author them.

See [CONTRIBUTING.md](CONTRIBUTING.md) for contributor workflow, quality bar, and community direction.

If you want to propose a new community learning path, start with [docs/community-recipes.md](docs/community-recipes.md).

## License

Primer is available under the [MIT License](LICENSE).
