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

echo "▶ Checking tokenizer output..."
TOKENS_JSON="$(python3 mini_lang.py tokens "sum = 1 + 2")"

python3 - <<'PY' "$TOKENS_JSON"
import json
import sys

data = json.loads(sys.argv[1])
assert isinstance(data, list), "tokens output must be a JSON list"
types = [token["type"] for token in data]
values = [token["value"] for token in data]
assert types == ["IDENT", "EQUAL", "NUMBER", "PLUS", "NUMBER"], types
assert values == ["sum", "=", "1", "+", "2"], values
PY

echo "▶ Checking whitespace handling..."
python3 mini_lang.py tokens "  sum   =   1 +   2  " >/dev/null

echo "▶ Checking invalid-character failure..."
set +e
ERROR_OUTPUT="$(python3 mini_lang.py tokens "1 @ 2" 2>&1)"
STATUS="$?"
set -e
[ "$STATUS" -ne 0 ] || fail "Expected non-zero exit for invalid character."
printf '%s\n' "$ERROR_OUTPUT" | grep -Eiq "invalid|unexpected|unknown" || fail "Expected helpful tokenizer error."

echo "✓ Milestone 01 check passed"
