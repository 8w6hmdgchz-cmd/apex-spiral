#!/bin/bash
# A2A 抓取 v9 - 简单版，只验证已存在的仓库
# 支持SSH/HTTPS/ghproxy多路 fallback
set -euo pipefail
LOG_DIR="/Users/lihongxin/.openclaw/workspace/a2a-resources"
CACHE="$LOG_DIR/cache"
mkdir -p "$LOG_DIR" "$CACHE"
TIMESTAMP=$(date "+%Y-%m-%d %H:%M")
echo "=== A2A抓取 v9 | $TIMESTAMP ===" >> "$LOG_DIR/fetcher.log"

append() { grep -qFx "$2" "$1" 2>/dev/null || echo "$2" >> "$1"; }

# 只放已验证存在的仓库
REPOS=(
  "microsoft/autogen|A2A协议"
  "openai/openai-agents-python|A2A协议"
  "mem0ai/mem0|记忆系统"
  "langchain-ai/langchain|长上下文"
  "langchain-ai/langgraph|长上下文"
  "noahshinn/reflexion|自我改进"
  "deap/deap|进化算法"
  "pyg-team/pytorch_geometric|图神经网络"
  "openai/spinningup|强化学习"
  "ray-project/ray|强化学习"
  "geek-ai/MAgent|多Agent系统"
  "facebookresearch/rlhive|强化学习"
  "QwenLM/Qwen-Agent|Agent框架"
  "THUDM/ChatGLM3|Agent框架"
  "anthropic/anthropic-sdk-python|工具调用"
)

# SSH timeout settings
export GIT_SSH_COMMAND="ssh -o ConnectTimeout=15 -o ServerAliveInterval=60 -o ServerAliveCountMax=3"

# clone_with_fallback <repo> <dest_dir>
# Returns 0 on success, non-zero on failure
clone_with_fallback() {
  local repo="$1" dest="$2"
  local methods=(
    "git clone --depth=1 git@github.com:${repo}.git ${dest}"
    "git clone --depth=1 https://github.com/${repo}.git ${dest}"
    "git clone --depth=1 https://ghproxy.com/https://github.com/${repo}.git ${dest}"
  )
  for cmd in "${methods[@]}"; do
    echo "  尝试: $cmd"
    if eval "$cmd" 2>/dev/null; then
      return 0
    fi
    echo "  失败，重试下一个方法..."
  done
  return 1
}

CLONED=0
for item in "${REPOS[@]}"; do
  repo="${item%%|*}"
  cat="${item##*|}"
  dir="$CACHE/${repo//\//_}"
  [ -d "$dir" ] && continue
  echo "克隆: $repo"
  if clone_with_fallback "$repo" "$dir"; then
    append "$LOG_DIR/pending.list" "$cat|$repo|github_clone"
    ((CLONED++))
  fi
done
echo "新增: $CLONED"