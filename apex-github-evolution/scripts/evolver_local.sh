#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."

echo "[1/4] EvoMap local audit"
python3 apex-github-evolution/scripts/evomap_audit.py

echo "[2/4] Rust token optimizer tests"
if [ -d apex_token_rs ]; then
  (cd apex_token_rs && cargo test --quiet)
fi

echo "[3/4] ClawG iteration smoke test"
if [ -x clawg-mvp/scripts/run_iteration.sh ]; then
  clawg-mvp/scripts/run_iteration.sh >/tmp/clawg_iteration.log
  tail -20 /tmp/clawg_iteration.log
fi

echo "[4/4] Unified engine smoke test"
if [ -x apex-unified-engine/scripts/run_engine.sh ]; then
  apex-unified-engine/scripts/run_engine.sh "APEX GitHub evolution safety audit" >/tmp/apex_engine.log
  cat /tmp/apex_engine.log
fi

echo "DONE: local-only evolution cycle complete. No external push/gist performed."
