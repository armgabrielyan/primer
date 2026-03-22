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
require_cmd mktemp

[ -d tests ] || fail "tests directory is missing."

PROGRAM_FILE="$(mktemp)"
trap 'rm -f "$PROGRAM_FILE"' EXIT

cat >"$PROGRAM_FILE" <<'EOF'
x = 2
y = x + 3
y * 4
EOF

echo "▶ Checking program execution..."
[ "$(python3 mini_lang.py run "$PROGRAM_FILE")" = "20" ] || fail "Expected program result 20."

echo "▶ Checking unknown-variable failure..."
cat >"$PROGRAM_FILE" <<'EOF'
y + 1
EOF

set +e
ERROR_OUTPUT="$(python3 mini_lang.py run "$PROGRAM_FILE" 2>&1)"
STATUS="$?"
set -e
[ "$STATUS" -ne 0 ] || fail "Expected non-zero exit for unknown variable."
printf '%s\n' "$ERROR_OUTPUT" | grep -Eiq "unknown|undefined|not found" || fail "Expected helpful unknown-variable error."

echo "▶ Running unit tests..."
python3 -m unittest discover -s tests -p 'test_*.py'

echo "✓ Milestone 04 check passed"
