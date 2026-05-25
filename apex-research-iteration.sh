#!/bin/bash
# APEX unified research engine iteration.
set -euo pipefail
WORKSPACE="/Users/lihongxin/.openclaw/workspace"
LOG="$WORKSPACE/research/apex/apex-research-iteration.log"
mkdir -p "$WORKSPACE/research/apex"
cd "$WORKSPACE"
QUESTION="${1:-APEX unified research engine periodic integrity check}"
{
  echo "=== APEX RESEARCH ITERATION $(date '+%Y-%m-%d %H:%M:%S %z') ==="
  cargo run --manifest-path crates/apex_research_engine/Cargo.toml -- --root research/apex --question "$QUESTION"
  latest="$(ls -td research/apex/projects/apex_research_* | head -1 || true)"
  if [ -n "$latest" ]; then
    echo "latest_project=$latest"
    cat "$latest/run.json"
  fi
  echo "=== APEX RESEARCH ITERATION END $(date '+%Y-%m-%d %H:%M:%S %z') ==="
} >> "$LOG" 2>&1

tail -100 "$LOG"
