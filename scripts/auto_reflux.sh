#!/usr/bin/env bash
# 自动回流管道 V10.1 — 猎食→吸收→编译→提交→Ω_dawn
# 替代手动操作，每次周期自动执行
#
# V10.1 新增 Ω_dawn 凌晨自进化逻辑：
#   - git 变更检查 → phi_tracker更新 → 提交并推送
#
# 每15-30分钟执行:
#   1. PHI追踪 (phi_tracker.sh) — 含V10.1公式计算
#   2. 猎食检查 (scavenge github priority)
#   3. V10.1 Ω_dawn 自进化
#   4. Git commit (含自进化日志)
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
  log "[1/5] PHI_RATIO追踪 (V10.1)..."

  if [ -f "$ROOT/scripts/phi_tracker.sh" ]; then
    # 优先调用 phi_tracker 的 V10.1 计算
    local v10_output
    v10_output=$(bash "$ROOT/scripts/phi_tracker.sh" 2>&1 | tee -a "$LOG" || true)
    
    # 检查是否使用了V10.1
    if echo "$v10_output" | grep -q "V10.1"; then
      log "  ✅ 使用 V10.1 ΔG 公式"
    else
      log "  ⚠️ 未使用 V10.1（二进制未编译或公式不符）"
    fi
  else
    log "  ⚠️ phi_tracker.sh 未就绪，跳过"
  fi
}

step_scavenge() {
  log "[2/5] 猎食检查..."
  SCAVENGE="$ROOT/apex-ene/scavenger/target/release/scavenge"
  if [ -f "$SCAVENGE" ]; then
    run_timeout 30 "$SCAVENGE" github priority 2>/dev/null >> "$LOG" || {
      log "  ⚠️ 猎食超时，降级为SSH快速可达性检测"
      for repo in "google-deepmind/alphafold" "deepseek-ai/DeepSeek-R1"; do
        git ls-remote "git@github.com:$repo" HEAD >> "$LOG" 2>&1 &
      done
      wait
    }
  fi
}

step_omega_dawn() {
  log "[3/5] Ω_dawn 凌晨自进化..."
  
  cd "$ROOT"
  
  # ── 3a. git 版本差异检测 ──
  local delta_version_diff=0.0
  local rho_sync_fail=0.0
  
  # 检查是否有未推送的commit
  local unpushed=$(git log --oneline origin/main..HEAD 2>/dev/null | wc -l | tr -d ' ')
  if [ -n "$unpushed" ] && [ "$unpushed" -gt 0 ]; then
    delta_version_diff=$(echo "scale=4; $unpushed / 10.0" | bc -l 2>/dev/null || echo "0.1")
    log "  📦 未推送commit数: $unpushed, δ_diff=${delta_version_diff}"
  fi

  # 检查是否有未提交的文件
  local unstaged=$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')
  if [ -n "$unstaged" ] && [ "$unstaged" -gt 0 ]; then
    log "  📝 未提交文件数: $unstaged"
  fi
  
  # 计算 Ω_dawn （简单bash版）
  local tau_auto_merge=0.8
  local git_sync_value=$(echo "scale=6; (1.0 - $delta_version_diff) * (1.0 - $rho_sync_fail) * $tau_auto_merge" | bc -l 2>/dev/null || echo "0.8")
  
  # ── 3b. 自动学习参数 ──
  local l_extract=0.85
  local g_generalize=0.75
  local s_summarize=0.8
  local t_time=0.5
  local auto_learn_value=$(echo "scale=6; $l_extract * $g_generalize * $s_summarize / ($t_time + 1.0)" | bc -l 2>/dev/null || echo "0.34")
  local dawn_omega=$(echo "scale=6; 1.0 * $git_sync_value * $auto_learn_value" | bc -l 2>/dev/null || echo "0.272")
  
  log "  Ω_dawn = $dawn_omega (git_sync=$git_sync_value, auto_learn=$auto_learn_value)"
  
  # ── 3c. 如果有 V10.1 binary，尝试获取精确结果 ──
  local APEXE="$ROOT/apex-ene/engine/target/release/apexe"
  if [ -f "$APEXE" ]; then
    local safe_result=$("$APEXE" calc-v10 --safe --json 2>/dev/null || echo "")
    if [ -n "$safe_result" ]; then
      local safe_g=$(echo "$safe_result" | python3 -c "import sys,json; print(json.load(sys.stdin).get('delta_g_safe','N/A'))" 2>/dev/null || echo "N/A")
      log "  ΔG_safe = $safe_g"
    fi
  fi
  
  # 记录Ω_dawn结果
  echo "{\"timestamp\":\"$(date -Iseconds)\",\"omega_dawn\":$dawn_omega,\"git_sync\":$git_sync_value,\"auto_learn\":$auto_learn_value,\"version\":\"V10.1\"}" >> "$ROOT/state/omega_dawn_history.jsonl" 2>/dev/null || true
}

step_git_commit() {
  log "[4/5] Git提交..."
  cd "$ROOT"
  
  # 提交 V10-core / phi_tracker / auto_reflux 变更
  GIT_FILES=(
    "EVOLUTION_MAP.md"
    "EVOLUTION_GENES_V2.md"
    "scripts/gist_backup.sh"
    "scripts/auto_reflux.sh"
    "scripts/phi_tracker.sh"
    "apex-ene/v10-core/src/lib.rs"
    "apex-ene/v10-core/Cargo.toml"
    "apex-ene/engine/src/main.rs"
    "apex-ene/engine/Cargo.toml"
    "state/omega_dawn_history.jsonl"
  )
  
  for f in "${GIT_FILES[@]}"; do
    [ -f "$f" ] && git add "$f" 2>/dev/null || true
  done
  
  # 检查是否有变更
  if git diff --cached --quiet 2>/dev/null; then
    log "  ✅ 无变更，跳过提交"
  else
    # 从Ω_dawn生成提交信息
    local dawn_log=$(tail -1 "$ROOT/state/omega_dawn_history.jsonl" 2>/dev/null | python3 -c "import sys,json; d=json.loads(sys.stdin.read()); print(f'Ω_dawn={d.get(\"omega_dawn\",\"?\")}')" 2>/dev/null || echo "")
    git commit -m "🔄 auto reflux V10.1 $(date '+%Y-%m-%d %H:%M') ${dawn_log}" 2>&1 | tee -a "$LOG"
    
    # 稳健推送
    if GIT_SSH_COMMAND="ssh -o ConnectTimeout=10" git fetch origin main 2>&1 | tee -a "$LOG"; then
      GIT_SSH_COMMAND="ssh -o ConnectTimeout=10" git rebase origin/main 2>&1 | tee -a "$LOG" || true
      if GIT_SSH_COMMAND="ssh -o ConnectTimeout=10" git push origin main 2>&1 | tee -a "$LOG"; then
        log "  ✅ Git推送成功"
      else
        log "  ⚠️ push失败, 尝试force-with-lease"
        GIT_SSH_COMMAND="ssh -o ConnectTimeout=10" git push --force-with-lease origin main 2>&1 | tee -a "$LOG" || log "  ❌ 推送失败"
      fi
    else
      log "  ⚠️ fetch失败, 跳过"
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
  log "🔄 自动回流管道 V10.1 启动"
  
  step_phi_tracker
  step_scavenge
  step_omega_dawn
  step_git_commit
  step_gist_backup
  
  log "✅ 回流完成"
  log "========================================="
}

main "$@"
