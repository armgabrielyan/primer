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

echo "▶ Checking CLI help..."
python3 task_cli.py --help >/dev/null

echo "▶ Checking empty list output..."
EMPTY_OUTPUT="$(python3 task_cli.py list)"
printf '%s\n' "$EMPTY_OUTPUT" | grep -iq "no tasks" || fail "Expected 'no tasks' message from list."

echo "▶ Adding first task..."
python3 task_cli.py add "buy milk" >/dev/null

[ -f tasks.json ] || fail "tasks.json was not created."

echo "▶ Validating tasks.json structure..."
python3 - <<'PY'
import json
from pathlib import Path

path = Path("tasks.json")
data = json.loads(path.read_text())
assert isinstance(data, list), "tasks.json must contain a list"
assert len(data) == 1, f"expected 1 task, found {len(data)}"
task = data[0]
assert task["id"] == 1, f"expected id 1, got {task['id']!r}"
assert task["title"] == "buy milk", f"unexpected title: {task['title']!r}"
assert task["done"] is False, f"expected done false, got {task['done']!r}"
PY

echo "▶ Checking list output..."
LIST_OUTPUT="$(python3 task_cli.py list)"
printf '%s\n' "$LIST_OUTPUT" | grep -q "buy milk" || fail "Expected task title in list output."
printf '%s\n' "$LIST_OUTPUT" | grep -q "1" || fail "Expected task id in list output."

echo "✓ Milestone 01 check passed"
