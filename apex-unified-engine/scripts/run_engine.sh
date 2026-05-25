#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
QUESTION="${*:-How can APEX improve reproducible local agent research workflows?}"
python3 py/orchestrator/run_engine.py "$QUESTION"
