# Milestone 01: Bootloader

## Goal

Build a BIOS-loadable 512-byte x86 boot sector that prints a boot message and can run in QEMU.

## What you'll build

- `boot.asm`: NASM source for a real-mode boot sector
- `Makefile`: build + run targets for `boot.bin`
- A boot sector binary that:
  - is exactly 512 bytes
  - ends with boot signature `0x55 0xAA`
  - prints `Hello from bootloader`

## Acceptance criteria

- `tests/check.sh` exits with code `0`
- `boot.bin` exists after build
- `boot.bin` size is exactly 512 bytes
- Byte `510` is `0x55`, byte `511` is `0xAA`
- QEMU serial output contains `Hello from bootloader`

## Files to create in project workspace

- `boot.asm`
- `Makefile`

## Suggested implementation notes

- Assemble with NASM in `bin` format
- Use BIOS teletype interrupt (`int 0x10`, `ah=0x0e`) to print characters
- Pad to 510 bytes and append boot signature bytes

## Resources

- https://wiki.osdev.org/Bootloader
- https://wiki.osdev.org/BIOS
