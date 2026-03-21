# Milestone 05: Memory Manager

## Goal

Implement an early physical memory manager and validate basic allocation/free behavior.

## What you'll build

- Frame allocator initialization
- `alloc`/`free` primitives (or equivalent API)
- Deterministic allocation test path with serial marker

## Acceptance criteria

- `tests/check.sh` exits `0`
- `make` succeeds and produces `boot.bin`
- Serial output includes marker: `Memory manager ready`

## Files to create or update in project workspace

- Memory manager source/header
- Kernel init path to run allocator sanity checks
- `Makefile`
