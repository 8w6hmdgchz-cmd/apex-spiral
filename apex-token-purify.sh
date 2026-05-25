#!/bin/bash
# APEX token purification smoke entrypoint.
set -euo pipefail
WORKSPACE="/Users/lihongxin/.openclaw/workspace"
LOG="$WORKSPACE/memory/apex-token-purify.log"
mkdir -p "$WORKSPACE/memory"
cd "$WORKSPACE"
{
  echo "=== APEX TOKEN PURIFY $(date '+%Y-%m-%d %H:%M:%S %z') ==="
  cargo run --manifest-path crates/apex_token_optimizer/Cargo.toml -- correct --x 100 --y 50 --sw 1920 --sh 1080 --iw 1000 --ih 500
  cargo run --manifest-path crates/apex_token_optimizer/Cargo.toml -- reserve --text 100 --imgs 1000,1200,900,800 --keep 3
  cargo run --manifest-path crates/apex_token_optimizer/Cargo.toml -- effort --total 100 --waste 25
  # Purify managed media download dir conservatively: keep latest 50 files; do not touch user workspace docs.
  if [ -d "/Users/lihongxin/.openclaw/media/qqbot/downloads" ]; then
    cargo run --manifest-path crates/apex_token_optimizer/Cargo.toml -- purify --dir /Users/lihongxin/.openclaw/media/qqbot/downloads --keep 50 || true
  fi
  echo "=== APEX TOKEN PURIFY END $(date '+%Y-%m-%d %H:%M:%S %z') ==="
} >> "$LOG" 2>&1

tail -60 "$LOG"
