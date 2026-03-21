#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "FAIL: $1" >&2
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "$1 is required but not installed."
}

run_with_timeout() {
  local seconds="$1"
  shift
  if command -v timeout >/dev/null 2>&1; then
    timeout "$seconds" "$@"
    return
  fi
  if command -v gtimeout >/dev/null 2>&1; then
    gtimeout "$seconds" "$@"
    return
  fi
  fail "timeout/gtimeout is required to run QEMU checks."
}

echo "▶ Checking dependencies..."
require_cmd make
require_cmd xxd
require_cmd grep
require_cmd qemu-system-i386

echo "▶ Building boot.bin..."
make boot.bin

if [ ! -f boot.bin ]; then
  fail "boot.bin was not created by make boot.bin."
fi

echo "▶ Checking size (512 bytes)..."
SIZE="$(wc -c < boot.bin | tr -d '[:space:]')"
[ "$SIZE" = "512" ] || fail "Expected boot.bin size 512, got $SIZE."

echo "▶ Checking boot signature at bytes 510-511..."
SIG="$(xxd -s 510 -l 2 -p boot.bin)"
[ "$SIG" = "55aa" ] || fail "Expected signature 55aa, got $SIG."

echo "▶ Running QEMU output check..."
QEMU_OUTPUT="$(run_with_timeout 10 qemu-system-i386 -drive format=raw,file=boot.bin -display none -serial stdio 2>&1 || true)"
printf '%s\n' "$QEMU_OUTPUT" | grep -q "Hello from bootloader" || fail "Expected output 'Hello from bootloader' not found."

echo "✓ Milestone 01 check passed"
