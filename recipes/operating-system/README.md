# Build Your Own Operating System

This recipe takes you from a 512-byte bootloader to a minimal shell through 8 milestones.

## Table of Contents

- [What you will build](#what-you-will-build)
- [Prerequisites](#prerequisites)
- [Why the project Makefile changes](#why-the-project-makefile-changes)
- [How to start](#how-to-start)
- [Milestones](#milestones)
- [Recipe validation](#recipe-validation)

## What you will build

- BIOS boot sector
- Kernel entry in C
- VGA output
- Interrupt handling
- Memory manager
- Round-robin scheduler
- Simple filesystem read path
- Minimal shell prompt

## Prerequisites

- `python3`: used by adapter generators and helper scripts
- `nasm`: needed from milestone 01
- `qemu-system-i386`: used to boot and verify the image
- `make`: used in the learner project workspace
- `i686-elf-gcc` and `i686-elf-ld`: needed from milestone 02 onward

Notes:

- Milestone 01 can be completed with `nasm`, `make`, and QEMU.
- Milestone 02+ requires a real 32-bit bare-metal cross-compiler.
- The milestone checks no longer require `timeout` or `gtimeout`; they fall back to `python3` if needed.

## Why the project Makefile changes

The `Makefile` is part of what the learner is building. It belongs in the learner project workspace because it defines how that project assembles, links, and runs.

The `primer` repository should stay a recipe library. The learner's source files, build artifacts, and project `Makefile` should live in a separate target directory.

That means updating the project `Makefile` is expected as the milestones grow. What should not happen is mixing those project build rules into the `primer` repo itself.

## How to start

From a separate target workspace:

```bash
PRIMER_ROOT=/path/to/primer
mkdir -p ~/workspace/my-os
cd ~/workspace/my-os
"$PRIMER_ROOT/scripts/validate-recipe" "$PRIMER_ROOT/recipes/operating-system"
"$PRIMER_ROOT/scripts/generate-claude-adapter" "$PRIMER_ROOT/recipes/operating-system" --output-dir .
```

Or for Codex:

```bash
"$PRIMER_ROOT/scripts/generate-codex-adapter" "$PRIMER_ROOT/recipes/operating-system" --output-dir .
```

That generates both:

- `AGENTS.md`
- `.agents/skills/primer-build/SKILL.md`
- `.agents/skills/primer-check/SKILL.md`
- `.agents/skills/primer-explain/SKILL.md`
- `.agents/skills/primer-status/SKILL.md`
- `.agents/skills/primer-next-milestone/SKILL.md`

Then in your AI tool, run milestone commands/skills:

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

1. Bootloader
2. Kernel entry
3. VGA output
4. Interrupts
5. Memory
6. Scheduler
7. Filesystem
8. Shell

## Recipe validation

Validate contract + structure:

```bash
scripts/validate-recipe recipes/operating-system
```
