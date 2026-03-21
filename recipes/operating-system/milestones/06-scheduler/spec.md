# Milestone 06: Process Scheduler

## Goal

Implement a basic round-robin scheduler and prove periodic context progression.

## What you'll build

- Minimal process/task structures
- Scheduler tick path and context switch progression
- Serial marker indicating scheduling is active

## Acceptance criteria

- `tests/check.sh` exits `0`
- `make` succeeds and produces `boot.bin`
- Serial output includes marker: `Scheduler tick`

## Files to create or update in project workspace

- Scheduler source/header
- Timer/interrupt integration for scheduler ticks
- `Makefile`
