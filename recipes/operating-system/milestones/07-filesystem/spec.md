# Milestone 07: Simple Filesystem

## Goal

Implement a minimal filesystem read path and verify file retrieval from disk image data.

## What you'll build

- On-disk structure parsing for a simple filesystem format
- Read-by-name (or read-by-index) file access path
- Serial marker confirming successful read

## Acceptance criteria

- `tests/check.sh` exits `0`
- `make` succeeds and produces `boot.bin`
- Serial output includes marker: `Filesystem read ok`

## Files to create or update in project workspace

- Filesystem parser/read source/header
- Disk image test fixture integration
- `Makefile`
