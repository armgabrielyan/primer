# Milestone 02: Kernel Entry Point

## Goal

Transfer control from bootloader assembly to a C kernel entrypoint and confirm execution in QEMU.

## What you'll build

- A boot flow that jumps from stage-1 boot code into kernel entry code
- Minimal kernel entry implementation in C
- Build wiring so `make` produces a bootable `boot.bin`

## Acceptance criteria

- `tests/check.sh` exits `0`
- `make` succeeds and produces `boot.bin`
- Serial output includes marker: `Hello from kernel`

## Files to create or update in project workspace

- `boot.asm`
- `kernel.c` (or equivalent kernel entry source)
- `linker.ld` (or equivalent linker script)
- `Makefile`
