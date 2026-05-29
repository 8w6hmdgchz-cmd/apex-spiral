#!/bin/bash
# A2A 抓取 v10 - 直接下载文件，不用 git clone
# 绕过 GitHub 克隆超时问题
set -uo pipefail
LOG_DIR="/Users/lihongxin/.openclaw/workspace/a2a-resources"
CACHE="$LOG_DIR/cache"
mkdir -p "$LOG_DIR" "$CACHE"
TIMESTAMP=$(date "+%Y-%m-%d %H:%M")
echo "=== A2A抓取 v10 | $TIMESTAMP ===" >> "$LOG_DIR/fetcher.log"

append() { grep -qFx "$2" "$1" 2>/dev/null || echo "$2" >> "$1"; }

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

# 直接下载文件（绕过 git clone）
download_files() {
  local repo="$1" dest="$2"
  local branches=("main" "master")
  
  mkdir -p "$dest"
  
  # 尝试下载 README（快速失败）
  for branch in "${branches[@]}"; do
    url="https://raw.githubusercontent.com/${repo}/${branch}/README.md"
    if curl -sf --max-time 10 -A 'Mozilla/5.0' "$url" -o "$dest/README.md" 2>/dev/null; then
      echo "README from $branch" > "$dest/SOURCE.txt"
      return 0
    fi
  done
  
  # 仓库可能不存在或为空，快速返回失败
  return 1
}

# 清理空目录
cleanup_empty() {
  for d in "$CACHE"/*/; do
    [ -d "$d" ] || continue
    if [ $(ls -A "$d" 2>/dev/null | wc -l) -eq 0 ]; then
      rmdir "$d" 2>/dev/null
    fi
  done
}

CLONED=0
FAILED=0

for item in "${REPOS[@]}"; do
  repo="${item%%|*}"
  cat="${item##*|}"
  dir="$CACHE/${repo//\//_}"
  
  # 跳过已有 README 的
  if [ -f "$dir/README.md" ]; then
    continue
  fi
  
  echo "下载: $repo"
  if download_files "$repo" "$dir"; then
    append "$LOG_DIR/pending.list" "$cat|$repo|v10_download"
    ((CLONED++))
  else
    ((FAILED++))
    echo "  失败: $repo"
  fi
done

cleanup_empty

echo "完成: 新增$CLONED 个, 失败$FAILED 个"
