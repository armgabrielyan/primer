# Agent Instructions: 01-bootloader

## Learner Track

Before writing code, explain:

- What the BIOS does before your boot sector runs
- Why boot code is loaded at `0x7c00`
- Why the final two bytes must be `0x55` and `0xAA`

Ask exactly one question before coding:

- "What do you think happens if the boot signature is missing?"

Then implement together and narrate each section:

- segment setup and entry
- print loop via BIOS interrupt
- padding and signature bytes

After `tests/check.sh` passes, ask the learner to intentionally break one acceptance criterion, re-run the check, and explain the failure.

## Builder Track

Implement a minimal boot sector and `Makefile` that satisfy all acceptance criteria in `spec.md`.

Execution rules:

- Run `tests/check.sh`
- If checks fail, fix and re-run until pass
- Keep commentary minimal unless user asks
