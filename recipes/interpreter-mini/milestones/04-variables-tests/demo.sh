#!/usr/bin/env bash
set -euo pipefail

PROGRAM_FILE="$(mktemp)"
trap 'rm -f "$PROGRAM_FILE"' EXIT

cat >"$PROGRAM_FILE" <<'EOF'
x = 2
y = x + 3
y * 4
EOF

python3 mini_lang.py run "$PROGRAM_FILE"
python3 -m unittest discover -s tests -p 'test_*.py'
