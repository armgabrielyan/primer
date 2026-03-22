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

[ -d tests ] || fail "tests directory is missing."

TASKS_BACKUP=""
if [ -f tasks.json ]; then
  TASKS_BACKUP="$(mktemp)"
  cp tasks.json "$TASKS_BACKUP"
fi
trap restore_tasks EXIT

rm -f tasks.json

echo "▶ Creating mixed-status tasks..."
python3 task_cli.py add "buy milk" >/dev/null
python3 task_cli.py add "read docs" >/dev/null
python3 task_cli.py add "write notes" >/dev/null
python3 task_cli.py done 2 >/dev/null

echo "▶ Checking pending filter..."
PENDING_OUTPUT="$(python3 task_cli.py list --status pending)"
printf '%s\n' "$PENDING_OUTPUT" | grep -q "buy milk" || fail "Expected pending task in pending filter output."
printf '%s\n' "$PENDING_OUTPUT" | grep -q "write notes" || fail "Expected pending task in pending filter output."
if printf '%s\n' "$PENDING_OUTPUT" | grep -q "read docs"; then
  fail "Completed task appeared in pending filter output."
fi

echo "▶ Checking done filter..."
DONE_OUTPUT="$(python3 task_cli.py list --status done)"
printf '%s\n' "$DONE_OUTPUT" | grep -q "read docs" || fail "Expected completed task in done filter output."
if printf '%s\n' "$DONE_OUTPUT" | grep -q "buy milk"; then
  fail "Pending task appeared in done filter output."
fi

echo "▶ Checking summary output..."
SUMMARY_OUTPUT="$(python3 task_cli.py summary)"
printf '%s\n' "$SUMMARY_OUTPUT" | grep -Eiq "total[^0-9]*3|3[^0-9]*total" || fail "Expected total count in summary."
printf '%s\n' "$SUMMARY_OUTPUT" | grep -Eiq "pending[^0-9]*2|2[^0-9]*pending" || fail "Expected pending count in summary."
printf '%s\n' "$SUMMARY_OUTPUT" | grep -Eiq "done[^0-9]*1|1[^0-9]*done" || fail "Expected done count in summary."

echo "▶ Running unit tests..."
python3 -m unittest discover -s tests -p 'test_*.py'

echo "✓ Milestone 04 check passed"
