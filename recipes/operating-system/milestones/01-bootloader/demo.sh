#!/usr/bin/env bash
set -euo pipefail

if [ ! -f boot.bin ]; then
  echo "boot.bin not found. Run: make boot.bin"
  exit 1
fi

echo "Launching QEMU. You should see: Hello from bootloader"
exec qemu-system-i386 -drive format=raw,file=boot.bin
