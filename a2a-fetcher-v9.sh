#!/bin/bash
# A2A 抓取 v9 - 简单版，只验证已存在的仓库
# 支持SSH/HTTPS/ghproxy多路 fallback
set -uo pipefail  # 去掉 -e，让循环可以继续
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
  "google/brax|强化学习"
  "ray-project/ray|强化学习"
  "geek-ai/MAgent|多Agent系统"
  "facebookresearch/rlhive|强化学习"
  "QwenLM/Qwen-Agent|Agent框架"
  "THUDM/ChatGLM3|Agent框架"
  "anthropics/anthropic-sdk-python|工具调用"
  "openai/openai-python|OpenAI SDK"
  "google/generativeai-python|Google生成AI"
  "meta-llama/llama|羊驼大模型"
  "mistralai/mistral-src|Mistral大模型"
  "deepseek-ai/DeepSeek|大模型"
  "tatsu-lab/stanford_alpaca|ALPACA指令微调"
  "lm-sys/FastChat|ChatGPT克隆"
  "Significant-Gravitas/AutoGPT|AutoGPT"
  "significant-gravitas/langchain-autogen|AutoGen集成"
  "jumpmind/reflect|自我反思"
  "EleutherAI/gpt-neox|大模型"
  "facebookresearch/llama|LLAMA模型"
  "huggingface/transformers|Transformers库"
  "huggingface/peft|PEFT微调"
  "lucidrains/PaLM-pytorch|PaLM实现"
  "rasbt/LLMs-from-scratch|from scratch实现"
  "Svaiter/awesome-evolutionary-algorithms|进化算法汇总"
  "facebookresearch/rlhive|强化学习hive"
  "k Savoy/rlplugins|RL插件"
)

# SSH 超时加长到30秒
export GIT_SSH_COMMAND="ssh -o ConnectTimeout=30 -o StrictHostKeyChecking=no -o ServerAliveInterval=60 -o ServerAliveCountMax=3"

# clone_with_fallback <repo> <dest_dir>
# 返回0成功，1失败
clone_with_fallback() {
  local repo="$1" dest="$2"
  local methods=(
    "git clone --depth=1 git@github.com:${repo}.git ${dest}"
    "git clone --depth=1 https://github.com/${repo}.git ${dest}"
    "git clone --depth=1 https://ghproxy.com/https://github.com/${repo}.git ${dest}"
  )
  for cmd in "${methods[@]}"; do
    echo "  尝试: $cmd"
    eval "$cmd" 2>/dev/null && return 0
    echo "  失败，尝试下一个..."
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