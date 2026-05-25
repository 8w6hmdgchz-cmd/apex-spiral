#!/bin/bash
# ClawG local agent training ecosystem iteration.
set -euo pipefail
WORKSPACE="/Users/lihongxin/.openclaw/workspace"
LOG="$WORKSPACE/bench/clawg/clawg-iteration.log"
mkdir -p "$WORKSPACE/bench/clawg"
cd "$WORKSPACE"
{
  echo "=== CLAWG ITERATION $(date '+%Y-%m-%d %H:%M:%S %z') ==="
  cargo run --manifest-path crates/clawg_bench/Cargo.toml -- --root bench/clawg
  latest="$(ls -t bench/clawg/results/*.json | head -1 || true)"
  if [ -n "$latest" ]; then
    echo "latest_result=$latest"
    cat "$latest"
  fi
  echo "=== CLAWG ITERATION END $(date '+%Y-%m-%d %H:%M:%S %z') ==="
} >> "$LOG" 2>&1

tail -80 "$LOG"
