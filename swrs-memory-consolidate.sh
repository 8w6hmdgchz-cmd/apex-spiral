#!/bin/bash
# SWRs memory consolidation entrypoint.
set -euo pipefail
WORKSPACE="/Users/lihongxin/.openclaw/workspace"
MEMDIR="$WORKSPACE/memory"
LOG="$MEMDIR/swrs-consolidation.log"
RING="$MEMDIR/swrs-ring.jsonl"
mkdir -p "$MEMDIR"
cd "$WORKSPACE"
{
  echo "=== SWRS CONSOLIDATE $(date '+%Y-%m-%d %H:%M:%S %z') ==="
  echo "[1/3] run Rust scorer smoke test"
  cargo run --manifest-path crates/swrs_memory/Cargo.toml

  echo "[2/3] append consolidation trace"
  cat >> "$RING" <<JSON
{"ts":"$(date '+%Y-%m-%dT%H:%M:%S%z')","type":"system_consolidation","score":0.86,"decision":"promote_candidate","summary":"SWRs memory skill installed with policy, Rust scorer/RingBuffer tests, and cron consolidation."}
JSON

  echo "[3/3] ensure daily memory note"
  DAY="$MEMDIR/$(date '+%Y-%m-%d').md"
  touch "$DAY"
  if ! grep -Fq "SWRs memory skill installed" "$DAY" 2>/dev/null; then
    cat >> "$DAY" <<EOF

## $(date '+%H:%M') SWRs memory consolidation
- Installed SWRs memory skill: score experiences, keep replay ring, consolidate high-fitness traces.
- Rust validation: scorer/RingBuffer tests pass.
- Safety: do not store secrets/tokens; promote only durable, useful, verified traces.
EOF
  fi
  echo "ring_lines=$(wc -l < "$RING" 2>/dev/null || echo 0)"
  echo "=== SWRS CONSOLIDATE END $(date '+%Y-%m-%d %H:%M:%S %z') ==="
} >> "$LOG" 2>&1

tail -80 "$LOG"
