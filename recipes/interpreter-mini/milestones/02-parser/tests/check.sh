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

echo "▶ Checking precedence parse..."
AST_ONE="$(python3 mini_lang.py parse "1 + 2 * 3")"
python3 - <<'PY' "$AST_ONE"
import json
import sys

tree = json.loads(sys.argv[1])
assert tree["type"] == "BinaryExpr", tree
assert tree["operator"] == "+", tree
assert tree["right"]["type"] == "BinaryExpr", tree
assert tree["right"]["operator"] == "*", tree
PY

echo "▶ Checking parentheses parse..."
AST_TWO="$(python3 mini_lang.py parse "(1 + 2) * 3")"
python3 - <<'PY' "$AST_TWO"
import json
import sys

tree = json.loads(sys.argv[1])
assert tree["type"] == "BinaryExpr", tree
assert tree["operator"] == "*", tree
assert tree["left"]["type"] == "BinaryExpr", tree
assert tree["left"]["operator"] == "+", tree
PY

echo "▶ Checking invalid syntax failure..."
set +e
ERROR_OUTPUT="$(python3 mini_lang.py parse "1 +" 2>&1)"
STATUS="$?"
set -e
[ "$STATUS" -ne 0 ] || fail "Expected non-zero exit for invalid syntax."
printf '%s\n' "$ERROR_OUTPUT" | grep -Eiq "parse|syntax|expected|unexpected" || fail "Expected helpful parser error."

echo "✓ Milestone 02 check passed"
