# Milestone 08: Userspace Shell

## Goal

Implement a minimal command shell running on top of the prior kernel subsystems.

## What you'll build

- Input loop that reads command lines
- Command dispatch for a small built-in command set
- Output flow integrated with terminal/VGA/serial

## Acceptance criteria

- `tests/check.sh` exits `0`
- `make` succeeds and produces `boot.bin`
- Serial output includes marker: `shell>`

## Files to create or update in project workspace

- Shell parser/dispatch source
- Userspace or kernel command execution path
- `Makefile`
