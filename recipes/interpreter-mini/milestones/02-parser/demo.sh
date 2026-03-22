#!/usr/bin/env bash
set -euo pipefail

python3 mini_lang.py parse "1 + 2 * 3"
python3 mini_lang.py parse "(1 + 2) * 3"
