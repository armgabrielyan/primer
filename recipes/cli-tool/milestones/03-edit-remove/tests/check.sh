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
python3 task_cli.py add "write notes" >/dev/null

echo "▶ Editing and removing tasks..."
python3 task_cli.py edit 2 "read the Primer docs" >/dev/null
python3 task_cli.py remove 1 >/dev/null

echo "▶ Validating resulting JSON..."
python3 - <<'PY'
import json
from pathlib import Path

data = json.loads(Path("tasks.json").read_text())
assert len(data) == 2, f"expected 2 tasks after removal, found {len(data)}"
ids = [task["id"] for task in data]
titles = [task["title"] for task in data]
assert 1 not in ids, f"task 1 should have been removed, ids={ids!r}"
assert 2 in ids and 3 in ids, f"expected remaining ids 2 and 3, ids={ids!r}"
assert "read the Primer docs" in titles, f"edited title missing: {titles!r}"
assert "write notes" in titles, f"remaining task missing: {titles!r}"
PY

echo "▶ Checking list output..."
LIST_OUTPUT="$(python3 task_cli.py list)"
printf '%s\n' "$LIST_OUTPUT" | grep -q "read the Primer docs" || fail "Expected edited title in list output."
printf '%s\n' "$LIST_OUTPUT" | grep -q "write notes" || fail "Expected remaining task in list output."
if printf '%s\n' "$LIST_OUTPUT" | grep -q "buy milk"; then
  fail "Removed task still appears in list output."
fi

echo "▶ Checking missing-id errors..."
set +e
EDIT_ERROR="$(python3 task_cli.py edit 99 nope 2>&1)"
EDIT_STATUS="$?"
REMOVE_ERROR="$(python3 task_cli.py remove 99 2>&1)"
REMOVE_STATUS="$?"
set -e
[ "$EDIT_STATUS" -ne 0 ] || fail "Expected non-zero exit for edit on missing id."
[ "$REMOVE_STATUS" -ne 0 ] || fail "Expected non-zero exit for remove on missing id."
printf '%s\n' "$EDIT_ERROR" | grep -Eiq "not found|missing|unknown" || fail "Expected helpful edit error message."
printf '%s\n' "$REMOVE_ERROR" | grep -Eiq "not found|missing|unknown" || fail "Expected helpful remove error message."

echo "✓ Milestone 03 check passed"
