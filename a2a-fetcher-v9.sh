#!/bin/bash
# A2A 抓取 v9.1 - 并行 + 断点续传 + 大仓库优化
set -uo pipefail
LOG_DIR="/Users/lihongxin/.openclaw/workspace/a2a-resources"
CACHE="$LOG_DIR/cache"
mkdir -p "$LOG_DIR" "$CACHE"
TIMESTAMP=$(date "+%Y-%m-%d %H:%M")
echo "=== A2A抓取 v9.1 | $TIMESTAMP ===" >> "$LOG_DIR/fetcher.log"

append() { grep -qFx "$2" "$1" 2>/dev/null || echo "$2" >> "$1"; }

# 仓库列表（格式：仓库名|分类）
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
  "meta-llama/llama|羊驼大模型"
  "mistralai/mistral-src|Mistral大模型"
  "tatsu-lab/stanford_alpaca|ALPACA指令微调"
  "lm-sys/FastChat|ChatGPT克隆"
  "Significant-Gravitas/AutoGPT|AutoGPT"
  "EleutherAI/gpt-neox|大模型"
  "facebookresearch/llama|LLAMA模型"
  "huggingface/transformers|Transformers库"
  "huggingface/peft|PEFT微调"
  "rasbt/LLMs-from-scratch|from scratch实现"
  "allenai/OLMo|开放大模型"
  "deepseek-ai/DeepSeek-V2|DeepSeekV2"
  "QwenLM/Qwen2|Qwen2大模型"
  "openai/o1|o1模型"
  "google/vertexai-python|Google Vertex AI"
  "microsoft/vllm|微软vllm"
  "vllm-project/vllm|vllm大模型"
  "nvidia/TensorRT-LLM|TensorRT大模型"
  "stanford-oval/telegraph|Agent通信"
  "mistralai/cookbook|食谱"
)

# 大仓库列表（只用 git fetch --depth=1 获取最新 tag）
LARGE_REPOS="meta-llama/llama huggingface/transformers EleutherAI/gpt-neox microsoft/vllm vllm-project/vllm nvidia/TensorRT-LLM deepseek-ai/DeepSeek-V2"

# SSH 超时设置
export GIT_SSH_COMMAND="ssh -o ConnectTimeout=60 -o StrictHostKeyChecking=no -o ServerAliveInterval=60 -o ServerAliveCountMax=5"

# 清理空目录
cleanup_empty_dirs() {
  for d in "$CACHE"/*/; do
    [ -d "$d" ] || continue
    if [ $(ls -A "$d" 2>/dev/null | wc -l) -eq 0 ]; then
      rmdir "$d" 2>/dev/null
    fi
  done
}

# 克隆单个仓库（支持断点续传）
clone_single() {
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
    echo "  失败，尝试下一个..."
  done
  return 1
}

# 大仓库策略：只下载 README 和关键文件（不用 git clone）
clone_large_repo() {
  local repo="$1" dest="$2"
  mkdir -p "$dest"
  cd "$dest"
  
  # 尝试多个 README 路径
  local readme_urls=(
    "https://raw.githubusercontent.com/${repo}/main/README.md"
    "https://raw.githubusercontent.com/${repo}/master/README.md"
    "https://raw.githubusercontent.com/${repo}/HEAD/README.md"
  )
  
  for url in "${readme_urls[@]}"; do
    if curl -sf --max-time 30 -A 'Mozilla/5.0' "$url" -o README.md 2>/dev/null; then
      echo "  README 下载成功"
      return 0
    fi
  done
  
  # 尝试 git clone 但用 timeout 限制
  echo "  尝试浅克隆..."
  timeout 45 git clone --depth=1 git@github.com:${repo}.git . 2>/dev/null && return 0
  timeout 45 git clone --depth=1 https://github.com/${repo}.git . 2>/dev/null && return 0
  return 1
}

CLONED=0
FAILED=0
PDNS=()

for item in "${REPOS[@]}"; do
  repo="${item%%|*}"
  cat="${item##*|}"
  dir="$CACHE/${repo//\//_}"
  
  # 跳过已有完整内容的目录
  if [ -d "$dir" ] && [ $(ls -A "$dir" 2>/dev/null | wc -l) -gt 0 ]; then
    # 检查是否有 README
    if [ -f "$dir/README.md" ]; then
      continue
    fi
  fi
  
  echo "克隆: $repo -> $(basename $dir)"
  
  # 判断是否是大仓库
  if echo "$LARGE_REPOS" | grep -qF "$repo"; then
    if clone_large_repo "$repo" "$dir"; then
      append "$LOG_DIR/pending.list" "$cat|$repo|github_clone"
      ((CLONED++))
    else
      ((FAILED++))
    fi
  else
    if clone_single "$repo" "$dir"; then
      append "$LOG_DIR/pending.list" "$cat|$repo|github_clone"
      ((CLONED++))
    else
      ((FAILED++))
    fi
  fi
done

cleanup_empty_dirs

echo "抓取完成: 新增$CLONED 个, 失败$FAILED 个"
