# Build Your Own Operating System

This recipe takes you from a 512-byte bootloader to a minimal shell through 8 milestones.

## Table of Contents

- [What you will build](#what-you-will-build)
- [Prerequisites](#prerequisites)
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

- `nasm`
- `gcc`
- `qemu-system-i386`
- `make`

## How to start

From repo root:

```bash
scripts/validate-recipe recipes/operating-system
scripts/generate-claude-adapter recipes/operating-system --output-dir .
```

Or for Codex:

```bash
scripts/generate-codex-adapter recipes/operating-system --output-dir .
```

Then in your AI tool, run milestone commands/tasks:

- `status`
- `check`
- `explain`
- `next-milestone`

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
