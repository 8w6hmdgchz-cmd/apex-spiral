#!/bin/bash
# A2A auto trigger: fetch -> absorb -> hunt, safe for cron/OpenClaw.
set -euo pipefail

WORKSPACE="/Users/lihongxin/.openclaw/workspace"
RES="$WORKSPACE/a2a-resources"
LOG="$RES/auto-trigger.log"
LOCK="$RES/auto-trigger.lock"
mkdir -p "$RES"

if [ -f "$LOCK" ]; then
  old_pid="$(cat "$LOCK" 2>/dev/null || true)"
  if [ -n "$old_pid" ] && kill -0 "$old_pid" 2>/dev/null; then
    echo "[$(date '+%Y-%m-%d %H:%M:%S %z')] SKIP: auto trigger already running pid=$old_pid" | tee -a "$LOG"
    exit 0
  fi
fi
echo $$ > "$LOCK"
trap 'rm -f "$LOCK"' EXIT INT TERM

cd "$WORKSPACE"
{
  echo "=== A2A AUTO START $(date '+%Y-%m-%d %H:%M:%S %z') ==="

  # Normalize legacy typo artifacts into canonical files, then keep typos as backups.
  for name in absorbed pending failed; do
    if [ -f "$RES/$name. list" ]; then
      cat "$RES/$name. list" >> "$RES/$name.list"
      sort -u "$RES/$name.list" -o "$RES/$name.list"
      mv "$RES/$name. list" "$RES/$name. list.bak.$(date +%s)"
    fi
    if [ -f "$RES/$name." ]; then
      cat "$RES/$name." >> "$RES/$name.list"
      sort -u "$RES/$name.list" -o "$RES/$name.list"
      mv "$RES/$name." "$RES/$name.bak.$(date +%s)"
    fi
  done

  echo "[1/3] fetch"
  bash "$WORKSPACE/a2a-fetcher-v9.sh" || echo "fetch exited nonzero; continuing to absorb existing pending/cache"

  echo "[2/3] sync cache -> pending"
  while IFS= read -r -d '' dir; do
    repo="$(basename "$dir" | sed 's/_/\//g')"
    if ! grep -Fq "$repo" "$RES/absorbed.list" 2>/dev/null && ! grep -Fq "$repo" "$RES/pending.list" 2>/dev/null; then
      printf '待分类|%s|cache_sync\n' "$repo" >> "$RES/pending.list"
      echo "synced: $repo"
    fi
  done < <(find "$RES/cache" -mindepth 1 -maxdepth 1 -type d -print0)

  sort -u "$RES/pending.list" -o "$RES/pending.list" 2>/dev/null || true

  echo "[3/3] absorb"
  bash "$WORKSPACE/a2a-resource-absorber.sh" || echo "absorber exited nonzero"

  echo "[4/3] hunt"
  python3 "$WORKSPACE/a2a-hunt-realstate.py" || echo "hunt exited nonzero"

  echo "counts: absorbed=$(wc -l < "$RES/absorbed.list" 2>/dev/null || echo 0) pending=$(wc -l < "$RES/pending.list" 2>/dev/null || echo 0) failed=$(wc -l < "$RES/failed.list" 2>/dev/null || echo 0) cache=$(find "$RES/cache" -mindepth 1 -maxdepth 1 -type d | wc -l | tr -d ' ')"
  echo "=== A2A AUTO END $(date '+%Y-%m-%d %H:%M:%S %z') ==="
} >> "$LOG" 2>&1

tail -80 "$LOG"
