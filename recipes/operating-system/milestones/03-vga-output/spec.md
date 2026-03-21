# Milestone 03: VGA Text Driver

## Goal

Implement VGA text-mode output from kernel code and keep serial output for automated verification.

## What you'll build

- A VGA text writer for direct memory-mapped output
- Kernel integration that prints a visible VGA message
- A serial marker for non-interactive check automation

## Acceptance criteria

- `tests/check.sh` exits `0`
- `make` succeeds and produces `boot.bin`
- Serial output includes marker: `VGA driver ready`

## Files to create or update in project workspace

- VGA driver source/header
- Kernel print path
- `Makefile`
