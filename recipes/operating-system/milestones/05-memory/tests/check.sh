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
require_cmd qemu-system-i386
require_cmd grep

echo "▶ Building..."
make

[ -f boot.bin ] || fail "boot.bin not found after build."

echo "▶ Running QEMU output check..."
QEMU_OUTPUT="$(run_with_timeout 10 qemu-system-i386 -drive format=raw,file=boot.bin -display none -serial stdio 2>&1 || true)"
printf '%s\n' "$QEMU_OUTPUT" | grep -q "Memory manager ready" || fail "Expected marker 'Memory manager ready' not found."

echo "✓ Milestone 05 check passed"
