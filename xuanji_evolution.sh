#!/bin/bash
# 璇玑超级进化脚本 - 带锁协调

LOCK_FILE="/tmp/xuanji_gist.lock"
PID_FILE="/tmp/xuanji_gist.pid"
GIST_DIR="/tmp/gist_fresh"
GIST_REPO="git@gist.github.com:57fa0d7fc0247f91f9bb744c253c13ff.git"

# 检查锁
check_lock() {
  if [ -f "$LOCK_FILE" ]; then
    LOCK_PID=$(cat "$PID_FILE" 2>/dev/null)
    if [ -n "$LOCK_PID" ] && kill -0 "$LOCK_PID" 2>/dev/null; then
      echo "[$(date '+%H:%M:%S')] Hermes持有锁，跳过本轮"
      return 1
    fi
    rm -f "$LOCK_FILE" "$PID_FILE"
  fi
  return 0
}

# 获取锁
acquire_lock() {
  echo $$ > "$PID_FILE"
  echo "$(date '+%Y-%m-%d %H:%M:%S')" > "$LOCK_FILE"
}

# 释放锁
release_lock() {
  rm -f "$LOCK_FILE" "$PID_FILE"
}

# 主流程
main() {
  check_lock || exit 0
  acquire_lock
  
  echo "[$(date '+%Y-%m-%d %H:%M:%S')] 璇玑开始进化..."
  
  # 拉取最新
  cd "$GIST_DIR"
  GIT_SSH_COMMAND="ssh -4" git pull origin main 2>/dev/null
  
  # 读取当前状态
  source score-state.env 2>/dev/null
  
  # GPT-5.5分析
  ANALYSIS=$(~/.openclaw/workspace/skills/hetu-luoshu/hetu_luoshu call gpt-5.5 "APEX状态: PSI_SELF=${PSI_SELF:-6.6}, PHI_RATIO=${PHI_RATIO:-1.0}。简短分析并给出1个改进建议。" --max-tokens 300 2>/dev/null)
  
  # 记录分析
  echo "{\"time\":\"$(date -I)\",\"PSI_SELF\":\"$PSI_SELF\",\"PHI_RATIO\":\"$PHI_RATIO\",\"analysis\":\"$ANALYSIS\"}" >> phi_history_璇玑.jsonl
  
  # 提交推送
  git add phi_history_璇玑.jsonl
  git commit -m "璇玑分析 $(date '+%H:%M:%S')" 2>/dev/null
  GIT_SSH_COMMAND="ssh -4" git push origin main 2>/dev/null
  
  echo "[$(date '+%H:%M:%S')] 璇玑完成"
  release_lock
}

main
