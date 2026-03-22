#!/usr/bin/env bash
set -euo pipefail

python3 task_cli.py add "buy milk"
python3 task_cli.py add "read docs"
python3 task_cli.py done 2
python3 task_cli.py list --status pending
python3 task_cli.py list --status done
python3 task_cli.py summary
python3 -m unittest discover -s tests -p 'test_*.py'
