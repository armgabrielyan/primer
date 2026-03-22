#!/usr/bin/env bash
set -euo pipefail

python3 task_cli.py add "buy milk"
python3 task_cli.py add "read docs"
python3 task_cli.py edit 2 "read the Primer docs"
python3 task_cli.py remove 1
python3 task_cli.py list
