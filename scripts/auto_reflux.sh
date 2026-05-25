#!/usr/bin/env bash
# 自动回流管道 — 猎食→吸收→编译→提交→EvoMap
# 替代手动操作，每次周期自动执行
#
# 每15分钟执行:
#   1. PHI追踪 (phi_tracker.sh)
#   2. 猎食检查 (scavenge github priority)
#   3. EvoMap更新
#   4. Git commit
#   5. Gist备份
#
# 验收: 无待处理资源 + git无未提交改动

set -Eeuo pipefail
IFS=$'\n\t'

ROOT="${ROOT:-/Users/lihongxin/.openclaw/workspace}"
LOG="$ROOT/auto_reflux.log"
LOCK_FILE="/tmp/xuanji_reflux.lock"

log() { 
  echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG"
}

run_timeout() {
  local seconds="$1"; shift
  if command -v timeout >/dev/null 2>&1; then
    timeout "$seconds" "$@"
  elif command -v gtimeout >/dev/null 2>&1; then
    gtimeout "$seconds" "$@"
  else
    perl -e 'alarm shift; exec @ARGV' "$seconds" "$@"
  fi
}

require() {
  command -v "$1" >/dev/null 2>&1 || { log "❌ missing: $1"; return 1; }
}

with_lock() {
  if ! mkdir "$LOCK_FILE" 2>/dev/null; then
    log "⏳ 另一个回流管道正在运行，跳过"
    exit 0
  fi
  trap 'rm -rf "$LOCK_FILE"' EXIT
}

step_phi_tracker() {
  log "[1/5] PHI_RATIO追踪..."
  if [ -f "$ROOT/scripts/phi_tracker.sh" ]; then
    bash "$ROOT/scripts/phi_tracker.sh" 2>&1 | tee -a "$LOG" || true
  else
    log "  ⚠️ phi_tracker.sh 未就绪，跳过"
  fi
}

step_scavenge() {
  log "[2/5] 猎食检查..."
  SCAVENGE="$ROOT/apex-ene/scavenger/target/release/scavenge"
  if [ -f "$SCAVENGE" ]; then
    # 快速检查优先列表
    run_timeout 10 "$SCAVENGE" github priority 2>/dev/null >> "$LOG" || log "  ⚠️ 猎食超时或失败"
  fi
}

step_evomap_update() {
  log "[3/5] EvoMap同步..."
  if [ -f "$ROOT/apex-github-evolution/scripts/evomap_audit.py" ]; then
    python3 "$ROOT/apex-github-evolution/scripts/evomap_audit.py" 2>&1 | tail -3 | tee -a "$LOG" || true
  fi
}

step_git_commit() {
  log "[4/5] Git提交..."
  cd "$ROOT"
  
  # 只提交 EvoMap/基因/脚本变更
  GIT_FILES=(
    "EVOLUTION_MAP.md"
    "EVOLUTION_GENES_V2.md"
    "scripts/gist_backup.sh"
    "scripts/auto_reflux.sh"
    "scripts/phi_tracker.sh"
  )
  
  for f in "${GIT_FILES[@]}"; do
    [ -f "$f" ] && git add "$f" 2>/dev/null || true
  done
  
  # 检查是否有变更
  if git diff --cached --quiet 2>/dev/null; then
    log "  ✅ 无变更，跳过提交"
  else
    git commit -m "🔄 auto reflux $(date '+%Y-%m-%d %H:%M')" 2>&1 | tee -a "$LOG"
    if git push origin main 2>&1 | tee -a "$LOG"; then
      log "  ✅ Git推送成功"
    else
      log "  ⚠️ Git推送需要拉取最新"
      git pull --rebase origin main 2>/dev/null
      git push origin main 2>&1 | tee -a "$LOG" || log "  ❌ 推送失败"
    fi
  fi
}

step_gist_backup() {
  log "[5/5] Gist备份..."
  if [ -f "$ROOT/scripts/gist_backup.sh" ]; then
    run_timeout 120 bash "$ROOT/scripts/gist_backup.sh" 2>&1 | tee -a "$LOG" || log "  ⚠️ Gist备份失败"
  fi
}

main() {
  cd "$ROOT"
  with_lock
  
  log "========================================="
  log "🔄 自动回流管道启动"
  
  step_phi_tracker
  step_scavenge
  step_evomap_update
  step_git_commit
  step_gist_backup
  
  log "✅ 回流完成"
  log "========================================="
}

main "$@"
