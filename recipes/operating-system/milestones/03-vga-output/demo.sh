#!/usr/bin/env bash
set -euo pipefail

[ -f boot.bin ] || { echo "boot.bin not found. Run: make"; exit 1; }
echo "Launching QEMU. Expected serial marker: VGA driver ready"
exec qemu-system-i386 -drive format=raw,file=boot.bin
