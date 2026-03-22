#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "FAIL: $1" >&2
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "$1 is required but not installed."
}

require_cmd python3

echo "▶ Checking arithmetic evaluation..."
[ "$(python3 mini_lang.py eval "1 + 2 * 3")" = "7" ] || fail "Expected 7 for 1 + 2 * 3."
[ "$(python3 mini_lang.py eval "(1 + 2) * 3")" = "9" ] || fail "Expected 9 for (1 + 2) * 3."
[ "$(python3 mini_lang.py eval "8 / 2 + 1")" = "5" ] || fail "Expected 5 for 8 / 2 + 1."

echo "▶ Checking invalid syntax failure..."
set +e
ERROR_OUTPUT="$(python3 mini_lang.py eval "1 +" 2>&1)"
STATUS="$?"
set -e
[ "$STATUS" -ne 0 ] || fail "Expected non-zero exit for invalid syntax."
printf '%s\n' "$ERROR_OUTPUT" | grep -Eiq "parse|syntax|expected|unexpected" || fail "Expected helpful evaluation error."

echo "✓ Milestone 03 check passed"
