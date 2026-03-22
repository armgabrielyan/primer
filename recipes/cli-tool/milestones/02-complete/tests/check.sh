#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "FAIL: $1" >&2
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "$1 is required but not installed."
}

restore_tasks() {
  if [ -n "${TASKS_BACKUP:-}" ] && [ -f "$TASKS_BACKUP" ]; then
    mv "$TASKS_BACKUP" tasks.json
  else
    rm -f tasks.json
  fi
}

require_cmd python3
require_cmd grep
require_cmd mktemp

TASKS_BACKUP=""
if [ -f tasks.json ]; then
  TASKS_BACKUP="$(mktemp)"
  cp tasks.json "$TASKS_BACKUP"
fi
trap restore_tasks EXIT

rm -f tasks.json

echo "▶ Creating sample tasks..."
python3 task_cli.py add "buy milk" >/dev/null
python3 task_cli.py add "read docs" >/dev/null

echo "▶ Marking first task done..."
python3 task_cli.py done 1 >/dev/null

echo "▶ Validating updated JSON..."
python3 - <<'PY'
import json
from pathlib import Path

data = json.loads(Path("tasks.json").read_text())
assert len(data) == 2, f"expected 2 tasks, found {len(data)}"
assert data[0]["id"] == 1 and data[0]["title"] == "buy milk"
assert data[0]["done"] is True, f"expected task 1 done, got {data[0]['done']!r}"
assert data[1]["id"] == 2 and data[1]["title"] == "read docs"
assert data[1]["done"] is False, f"expected task 2 pending, got {data[1]['done']!r}"
PY

echo "▶ Checking list output..."
LIST_OUTPUT="$(python3 task_cli.py list)"
printf '%s\n' "$LIST_OUTPUT" | grep -q "buy milk" || fail "Expected first task in list output."
printf '%s\n' "$LIST_OUTPUT" | grep -q "read docs" || fail "Expected second task in list output."
printf '%s\n' "$LIST_OUTPUT" | grep -Eiq "done|complete|x|\\[x\\]" || fail "Expected a visible completed status."
printf '%s\n' "$LIST_OUTPUT" | grep -Eiq "pending|todo|open|\\[ \\]" || fail "Expected a visible pending status."

echo "▶ Checking missing-id error handling..."
set +e
ERROR_OUTPUT="$(python3 task_cli.py done 99 2>&1)"
STATUS="$?"
set -e
[ "$STATUS" -ne 0 ] || fail "Expected non-zero exit when completing a missing task."
printf '%s\n' "$ERROR_OUTPUT" | grep -Eiq "not found|missing|unknown" || fail "Expected helpful error for missing task id."

echo "✓ Milestone 02 check passed"
