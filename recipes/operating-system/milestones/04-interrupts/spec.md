# Milestone 04: GDT + Interrupts

## Goal

Configure interrupt handling (IDT/IRQ path) and verify keyboard interrupt flow.

## What you'll build

- Core interrupt descriptor setup
- PIC/IRQ setup for keyboard interrupt path
- Interrupt handler wiring with serial diagnostic output

## Acceptance criteria

- `tests/check.sh` exits `0`
- `make` succeeds and produces `boot.bin`
- Serial output includes marker: `IRQ keyboard ready`

## Files to create or update in project workspace

- Interrupt descriptor/handler sources
- Keyboard IRQ handler integration
- `Makefile`
